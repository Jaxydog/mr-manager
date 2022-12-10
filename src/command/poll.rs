use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroI64,
};

use rand::{thread_rng, Rng};

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
pub const CM_REMOVE: &str = formatcp!("{NAME}_remove");
pub const CM_RESULTS: &str = formatcp!("{NAME}_results");
pub const CM_LAST: &str = formatcp!("{NAME}_last");
pub const CM_NEXT: &str = formatcp!("{NAME}_next");

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
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Active(pub BTreeSet<(GuildId, UserId)>);

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
    pub fn archive_req(&self) -> Result<Req<Self>> {
        let anchor = self.anchor()?;
        let dir = format!("{NAME}/{}/{}", anchor.guild, self.user);

        Ok(Req::new(dir, anchor.message.to_string()))
    }
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

    pub fn as_remove_message(&self) -> CreateInteractionResponseMessage {
        let embed = CreateEmbed::new().color(BOT_COLOR).title("Remove Inputs");
        let mut message = CreateInteractionResponseMessage::new().embed(embed);

        for (index, input) in self.inputs.iter().enumerate() {
            let label = match input {
                Input::RandomRaffle => continue,
                Input::MultipleChoice(i) => &i.label,
                Input::TextResponse(i) => &i.label,
            };

            let button = CreateButton::new(CustomId::new(CM_REMOVE).arg(self.user).arg(index))
                .label(label)
                .style(ButtonStyle::Danger);

            message = message.button(button);
        }

        message.ephemeral(true)
    }

    pub async fn send(&mut self, http: &Http, channel: ChannelId, force: bool) -> Result<()> {
        let mut active = Active::read(()).await.unwrap_or_default();
        let embed = self.try_as_embed(http, ()).await?;
        let mut message = CreateMessage::new().embed(embed);

        for button in self.as_buttons(false, ()) {
            message = message.button(button);
        }

        if let Ok(anchor) = self.anchor() {
            if force {
                anchor.to_message(http).await?.delete(http).await?;
            } else {
                return Err(Error::Other("The poll has already been sent"));
            }
        }

        let message = channel.send_message(http, message).await?;
        self.anchor = Some(Anchor::try_from(message)?);
        active.0.insert((self.anchor()?.guild, self.user));

        self.write().await?;
        active.write().await
    }
    pub async fn close(mut self, http: &Http) -> Result<()> {
        let mut active = Active::read(()).await.unwrap_or_default();
        active.0.remove(&(self.anchor()?.guild, self.user));

        let mut message = self.anchor()?.to_message(http).await?;
        let mut edit = EditMessage::new().components(vec![]);

        for button in self.as_buttons(true, ()) {
            edit = edit.button(button);
        }

        message.edit(http, edit).await?;
        self.output = Some(self.__gen_output());

        let results = self.__try_as_result_message(http, false, &message).await?;
        self.anchor()?.channel.send_message(http, results).await?;

        self.archive_req()?.write(&self).await?;
        self.remove().await?;
        active.write().await
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

    fn __gen_output_choice(&self) -> Output {
        let total = self.replies.len();
        let mut entries = vec![];

        for (index, input) in self.inputs.iter().enumerate() {
            let Input::MultipleChoice(_) = input else {
                continue;
            };
            let mut entry = MultipleChoiceOutputEntry {
                votes: 0,
                users: vec![],
            };

            for (user, reply) in &self.replies {
                let Reply::MultipleChoice(reply) = reply else {
                    continue;
                };
                if reply.index != index {
                    continue;
                }

                entry.votes += 1;
                entry.users.push(*user);
            }

            entries.push(entry);
        }

        Output::MultipleChoice(MultipleChoiceOutput { total, entries })
    }
    fn __gen_output_raffle(&self) -> Output {
        let users: Vec<_> = self.replies.keys().copied().collect();
        let winner = users[thread_rng().gen_range(0..users.len())];

        Output::RandomRaffle(RandomRaffleOutput { winner, users })
    }
    fn __gen_output_text(&self) -> Output {
        let total = self.replies.len();
        let answers = self
            .replies
            .clone()
            .into_iter()
            .map_while(|(user, reply)| {
                let Reply::TextResponse(reply) = reply else {
                return None;
            };

                Some((user, reply.answers))
            })
            .collect();

        Output::TextResponse(TextResponseOutput { total, answers })
    }
    fn __gen_output(&self) -> Output {
        match self.kind {
            Kind::MultipleChoice => self.__gen_output_choice(),
            Kind::RandomRaffle => self.__gen_output_raffle(),
            Kind::TextResponse => self.__gen_output_text(),
        }
    }

    async fn __try_as_result_embed(&self, http: &Http) -> Result<CreateEmbed> {
        let user = http.get_user(self.user).await?;
        let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
        let mut embed = CreateEmbed::new()
            .author(author)
            .color(user.accent_colour.unwrap_or(BOT_COLOR));

        if self.content.hide_results {
            embed = embed.description("Results are only viewable by the poll author!");
        };

        Ok(embed)
    }
    #[allow(clippy::unused_self)]
    fn __as_result_buttons(&self, disabled: bool) -> Vec<CreateButton> {
        vec![CreateButton::new(CustomId::new(CM_RESULTS))
            .disabled(disabled)
            .emoji('📊')
            .label("View Results")
            .style(ButtonStyle::Primary)]
    }
    async fn __try_as_result_message(
        &self,
        http: &Http,
        disabled: bool,
        reply: &Message,
    ) -> Result<CreateMessage> {
        let mut message = CreateMessage::new().embed(self.__try_as_result_embed(http).await?);

        for button in self.__as_result_buttons(disabled) {
            message = message.button(button);
        }

        Ok(message.reference_message(reply))
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

    async fn try_as_embed(&self, http: &Http, _: Self::Args<'_>) -> Result<CreateEmbed> {
        let user = http.get_user(self.user).await?;
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

pub async fn check(http: &Http) -> Result<()> {
    todo!()
}

pub async fn run_command(_http: &Http, _cmd: &CommandInteraction) -> Result<()> {
    Ok(())
}
pub async fn run_component(_http: &Http, _cpn: &mut ComponentInteraction) -> Result<()> {
    Ok(())
}
pub async fn run_modal(_http: &Http, _mdl: &ModalInteraction) -> Result<()> {
    Ok(())
}
