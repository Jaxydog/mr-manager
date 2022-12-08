use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroI64,
};

use crate::prelude::*;

pub use self::input::*;
pub use self::output::*;
pub use self::reply::*;

pub mod input;
pub mod output;
pub mod reply;

pub const NAME: &str = "poll";

pub const CM_CHOICE: &str = formatcp!("{NAME}_choice");
pub const CM_RAFFLE: &str = formatcp!("{NAME}_raffle");
pub const CM_TEXT: &str = formatcp!("{NAME}_text");

pub const MD_SUBMIT: &str = formatcp!("{NAME}_submit");

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    MultipleChoice,
    RandomRaffle,
    TextResponse,
}

impl Kind {
    pub const fn max_inputs(&self) -> usize {
        match self {
            Self::MultipleChoice => 10,
            Self::RandomRaffle => 0,
            Self::TextResponse => 5,
        }
    }
}

impl TryFrom<i32> for Kind {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
        match value {
            0 => Ok(Self::MultipleChoice),
            1 => Ok(Self::TextResponse),
            _ => Err(Error::InvalidId(Value::Data, value.to_string())),
        }
    }
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let emoji = match self {
            Self::MultipleChoice => '🔢',
            Self::RandomRaffle => '🎲',
            Self::TextResponse => '📝',
        };

        write!(f, "{emoji} ")?;

        for (index, character) in format!("{self:?}").char_indices() {
            if character.is_uppercase() && index != 0 {
                write!(f, " ")?;
            }

            write!(f, "{character}")?;
        }

        Ok(())
    }
}

#[repr(transparent)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Active(pub BTreeSet<UserId>);

impl NewReq for Active {
    type Args = ();

    fn new_req(_: Self::Args) -> Req<Self> {
        Req::new(NAME, ".active")
    }
}

impl AsReq for Active {
    fn as_req(&self) -> Req<Self> {
        Self::new_req(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Content {
    pub title: String,
    pub description: String,
    pub hours: NonZeroI64,
    pub hide_members: bool,
    pub hide_results: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Poll {
    pub user: UserId,
    pub kind: Kind,
    pub inputs: Vec<Input>,
    pub replies: BTreeMap<UserId, Reply>,
    pub content: Content,
    anchor: Option<Anchor>,
    output: Option<Output>,
}

impl Poll {
    pub fn closes_at(&self) -> TimeString {
        let ms = self.content.hours.get() * 60 * 60 * 1000;
        let base = self.anchor().map_or_else(
            |_| Utc::now().timestamp_millis(),
            |anchor| anchor.message.created_at().timestamp_millis(),
        );

        TimeString::new(base + ms)
    }
    pub fn output(&self) -> Result<&Output> {
        self.output
            .as_ref()
            .ok_or(Error::MissingValue(Value::Other("Output")))
    }

    fn __add_buttons_choice(&self, buttons: &mut Vec<CreateButton>, disabled: bool) {
        for (index, input) in self.inputs.iter().enumerate().take(25) {
            let Input::MultipleChoice(input) = input else {
				continue;
			};

            buttons.push(input.as_button(disabled, (Kind::MultipleChoice, self.user, index)));
        }
    }
    fn __add_buttons_raffle(&self, buttons: &mut Vec<CreateButton>, disabled: bool) {
        buttons.push(
            CreateButton::new(CustomId::new(CM_RAFFLE).arg(self.user))
                .disabled(disabled)
                .emoji('🎲')
                .label("Enter Raffle")
                .style(ButtonStyle::Primary),
        );
    }
    fn __add_buttons_text(&self, buttons: &mut Vec<CreateButton>, disabled: bool) {
        buttons.push(
            CreateButton::new(CustomId::new(CM_TEXT).arg(self.user))
                .disabled(disabled)
                .emoji('📩')
                .label("Submit Response")
                .style(ButtonStyle::Primary),
        );
    }
}

impl Anchored for Poll {
    fn anchor(&self) -> Result<Anchor> {
        self.anchor.ok_or(Error::MissingValue(Value::Anchor))
    }
}

impl NewReq for Poll {
    type Args = (GuildId, UserId);

    fn new_req((guild, user): Self::Args) -> Req<Self> {
        Req::new(format!("{NAME}/{guild}"), user.to_string())
    }
}

impl TryAsReq for Poll {
    fn try_as_req(&self) -> Result<Req<Self>> {
        Ok(Self::new_req((self.anchor()?.guild, self.user)))
    }
}

#[async_trait]
impl TryAsEmbedAsync for Poll {
    type Args<'a> = ();

    async fn try_as_embed(&self, ctx: &Context, _: Self::Args<'_>) -> Result<CreateEmbed> {
        let user = ctx.http.get_user(self.user).await?;
        let author = CreateEmbedAuthor::new(user.tag()).url(user.face());
        let footer = CreateEmbedFooter::new(format!("Inputs: {}", self.inputs.len()));

        let mut description = format!("**Format:** {}\n", self.kind);
        description.push_str(&format!("**Closes:** {}\n\n", self.closes_at()));

        if self.content.hide_members {
            description.push_str("*Members are hidden*\n");
        } else {
            description.push_str("*Members are NOT hidden*\n");
        }

        if self.content.hide_results {
            description.push_str("*Results are hidden*\n");
        } else {
            description.push_str("*Results are NOT hidden*\n");
        }

        description.push_str(&format!("\n> {}", self.content.description));

        Ok(CreateEmbed::new()
            .author(author)
            .color(user.accent_colour.unwrap_or(BOT_COLOR))
            .description(description)
            .footer(footer)
            .thumbnail(user.face())
            .title(&self.content.title))
    }
}

impl AsButtonVec for Poll {
    type Args<'a> = ();

    fn as_buttons(&self, disabled: bool, _: Self::Args<'_>) -> Vec<CreateButton> {
        let mut buttons = vec![];

        match self.kind {
            Kind::MultipleChoice => self.__add_buttons_choice(&mut buttons, disabled),
            Kind::RandomRaffle => self.__add_buttons_raffle(&mut buttons, disabled),
            Kind::TextResponse => self.__add_buttons_text(&mut buttons, disabled),
        }

        buttons
    }
}

impl TryAsModal for Poll {
    type Args<'a> = ();

    fn try_as_modal(&self, _: Self::Args<'_>) -> Result<CreateModal> {
        if self.kind != Kind::TextResponse {
            return Err(Error::InvalidValue(Value::Data, self.kind.to_string()));
        }

        let mut components = vec![];

        for (index, input) in self.inputs.iter().enumerate().take(5) {
            let Input::TextResponse(input) = input else {
                continue;
            };

            components.push(CreateActionRow::InputText(CreateInputText::new(
                InputTextStyle::Short,
                &input.label,
                index.to_string(),
            )));
        }

        if components.is_empty() {
            return Err(Error::Other("You must provide at least one input"));
        }

        let custom_id = CustomId::new(MD_SUBMIT).arg(self.user.to_string());

        Ok(CreateModal::new(custom_id, "Submit Response").components(components))
    }
}

pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .default_member_permissions(Permissions::SEND_MESSAGES)
        .description("Create or manage polls")
        .dm_permission(false)
}

pub async fn run_command(_ctx: &Context, _cmd: &CommandInteraction) -> Result<()> {
    Ok(())
}
pub async fn run_component(_ctx: &Context, _cpn: &mut ComponentInteraction) -> Result<()> {
    Ok(())
}
pub async fn run_modal(_ctx: &Context, _mdl: &ModalInteraction) -> Result<()> {
    Ok(())
}
