use std::num::NonZeroI64;

use serenity::all::InputTextStyle;

use crate::prelude::*;

pub const NAME: &str = "poll";

pub const MULTIPLE_CHOICE: &str = formatcp!("{NAME}_choice");
pub const FREE_RESPONSE: &str = formatcp!("{NAME}_free");
pub const RANDOM_WINNER: &str = formatcp!("{NAME}_winner");

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct InputId(u8);

impl std::fmt::Display for InputId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    MultipleChoice,
    FreeResponse,
    RandomWinner,
}

impl Kind {
    #[must_use]
    pub const fn max_inputs(self) -> usize {
        match self {
            Self::MultipleChoice => 25,
            Self::FreeResponse => 5,
            Self::RandomWinner => 4,
        }
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, character) in format!("{self:?}").char_indices() {
            if character.is_uppercase() && index > 0 {
                write!(f, " ")?;
            }

            write!(f, "{character}")?;
        }

        Ok(())
    }
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub enum Input {
    MultipleChoice {
        input_id: InputId,
        label: String,
        icon: ReactionType,
    },
    FreeResponse {
        input_id: InputId,
        label: String,
        placeholder: String,
    },
    RandomWinner {
        input_id: InputId,
        label: String,
        icon: ReactionType,
    },
}

impl Input {
    #[must_use]
    pub const fn kind(&self) -> Kind {
        match self {
            Self::MultipleChoice { .. } => Kind::MultipleChoice,
            Self::FreeResponse { .. } => Kind::FreeResponse,
            Self::RandomWinner { .. } => Kind::RandomWinner,
        }
    }
    #[must_use]
    pub const fn input_id(&self) -> InputId {
        match self {
            Self::MultipleChoice { input_id, .. }
            | Self::FreeResponse { input_id, .. }
            | Self::RandomWinner { input_id, .. } => *input_id,
        }
    }
    #[must_use]
    pub fn label(&self) -> &str {
        match self {
            Self::MultipleChoice { label, .. }
            | Self::FreeResponse { label, .. }
            | Self::RandomWinner { label, .. } => label,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub enum Reply {
    MultipleChoice {
        user_id: UserId,
        input_id: InputId,
    },
    FreeResponse {
        user_id: UserId,
        response: Vec<(InputId, String)>,
    },
    RandomWinner {
        user_id: UserId,
        input_id: InputId,
    },
}

impl Reply {
    #[must_use]
    pub const fn kind(&self) -> Kind {
        match self {
            Self::MultipleChoice { .. } => Kind::MultipleChoice,
            Self::FreeResponse { .. } => Kind::FreeResponse,
            Self::RandomWinner { .. } => Kind::RandomWinner,
        }
    }
    #[must_use]
    pub const fn user_id(&self) -> UserId {
        match self {
            Self::MultipleChoice { user_id, .. }
            | Self::FreeResponse { user_id, .. }
            | Self::RandomWinner { user_id, .. } => *user_id,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub enum Output {
    MultipleChoice {},
    FreeResponse {},
    RandomWinner {},
}

impl Output {
    #[must_use]
    pub const fn kind(&self) -> Kind {
        match self {
            Self::MultipleChoice { .. } => Kind::MultipleChoice,
            Self::FreeResponse { .. } => Kind::FreeResponse,
            Self::RandomWinner { .. } => Kind::RandomWinner,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub title: String,
    pub description: String,
    pub hours: NonZeroI64,
    kind: Kind,
    hide_output: bool,
    hide_users: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Poll {
    pub user_id: UserId,

    pub content: Content,
    pub inputs: Vec<Input>,
    pub replies: Vec<Reply>,

    anchor: Option<Anchor>,
    output: Option<Output>,
}

impl Poll {
    pub const ERR_SENT: Error = Error::Other("The poll has already been sent");
    pub const ERR_UNSENT: Error = Error::Other("The poll has not been sent");
    pub const ERR_ACTIVE: Error = Error::Other("The poll is still active");
    pub const ERR_INACTIVE: Error = Error::Other("The poll is not active");
    pub const ERR_ARCHIVED: Error = Error::Other("The poll is archived");
    pub const ERR_UNARCHIVED: Error = Error::Other("The poll is not archived");
    pub const ERR_KIND: Error = Error::Other("The poll is an invalid type");

    #[must_use]
    pub fn archive_req(user_id: UserId, message_id: MessageId) -> Req<Self> {
        Req::new(format!("{NAME}/{user_id}"), message_id.to_string())
    }

    pub fn anchor(&self) -> Result<Anchor> {
        self.anchor.ok_or(Self::ERR_UNSENT)
    }
    pub fn output(&self) -> Result<&Output> {
        self.output.as_ref().ok_or(Self::ERR_ACTIVE)
    }
    #[must_use]
    pub const fn hides_users(&self) -> bool {
        self.content.hide_users
    }
    #[must_use]
    pub const fn hides_results(&self) -> bool {
        self.content.hide_output
    }

    #[must_use]
    pub fn is_kind(&self, kind: Kind) -> bool {
        self.content.kind == kind
    }
    #[must_use]
    pub const fn is_anchored(&self) -> bool {
        self.anchor.is_some()
    }
    #[must_use]
    pub const fn is_unanchored(&self) -> bool {
        self.anchor.is_none()
    }
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.is_anchored() && self.output.is_none()
    }
    #[must_use]
    pub const fn is_inactive(&self) -> bool {
        self.is_unanchored() || self.output.is_some()
    }
    pub async fn is_archived(&self) -> bool {
        let Ok(anchor) = self.anchor() else {
			return false;
		};

        Self::archive_req(self.user_id, anchor.message_id)
            .exists()
            .await
    }
    pub async fn is_unarchived(&self) -> bool {
        self.as_req().exists().await
    }

    fn __ensure(c: bool, e: Error) -> Result<()> {
        c.then_some(()).ok_or(e)
    }
    pub fn ensure_kind(&self, kind: Kind) -> Result<()> {
        Self::__ensure(self.is_kind(kind), Self::ERR_KIND)
    }
    pub fn ensure_anchored(&self) -> Result<()> {
        Self::__ensure(self.is_anchored(), Self::ERR_UNSENT)
    }
    pub fn ensure_unanchored(&self) -> Result<()> {
        Self::__ensure(self.is_unanchored(), Self::ERR_SENT)
    }
    pub fn ensure_active(&self) -> Result<()> {
        Self::__ensure(self.is_active(), Self::ERR_INACTIVE)
    }
    pub fn ensure_inactive(&self) -> Result<()> {
        Self::__ensure(self.is_inactive(), Self::ERR_ACTIVE)
    }
    pub async fn ensure_archived(&self) -> Result<()> {
        Self::__ensure(self.is_archived().await, Self::ERR_UNARCHIVED)
    }
    pub async fn ensure_unarchived(&self) -> Result<()> {
        Self::__ensure(self.is_unarchived().await, Self::ERR_ARCHIVED)
    }

    pub async fn archive(&self) -> Result<()> {
        self.ensure_unarchived().await?;

        let anchor = self.anchor()?;
        let request = Self::archive_req(self.user_id, anchor.message_id);

        request.write(self).await?;
        self.as_req().remove().await?;
        Ok(())
    }
    pub async fn unarchive(&self) -> Result<()> {
        self.ensure_archived().await?;

        let anchor = self.anchor()?;
        let request = Self::archive_req(self.user_id, anchor.message_id);

        self.as_req().write(self).await?;
        request.remove().await?;
        Ok(())
    }

    #[allow(clippy::needless_pass_by_value)]
    async fn __to_buttons_multiple_choice(
        &self,
        _: &Context,
        disabled: <Self as ToButtons>::Args,
    ) -> Result<Vec<CreateButton>> {
        let max = self.content.kind.max_inputs();
        let mut buttons = vec![];

        for input in self.inputs.iter().take(max) {
            let Input::MultipleChoice { input_id, label, icon } = input else {
				continue;
			};

            let custom_id = format!("{MULTIPLE_CHOICE};{};{input_id}", self.user_id);
            let button = CreateButton::new(custom_id)
                .disabled(disabled)
                .emoji(icon.clone())
                .label(label)
                .style(ButtonStyle::Secondary);

            buttons.push(button);
        }

        Ok(buttons)
    }
    #[allow(clippy::needless_pass_by_value)]
    async fn __to_buttons_free_response(
        &self,
        _: &Context,
        disabled: <Self as ToButtons>::Args,
    ) -> Result<Vec<CreateButton>> {
        let custom_id = format!("{FREE_RESPONSE};{}", self.user_id);

        Ok(vec![CreateButton::new(custom_id)
            .disabled(disabled)
            .emoji('📨')
            .label("Submit a Response")
            .style(ButtonStyle::Primary)])
    }
    #[allow(clippy::needless_pass_by_value)]
    async fn __to_buttons_random_winner(
        &self,
        _: &Context,
        disabled: <Self as ToButtons>::Args,
    ) -> Result<Vec<CreateButton>> {
        let max = self.content.kind.max_inputs();
        let mut buttons = vec![];

        for input in self.inputs.iter().take(max) {
            let Input::RandomWinner { input_id, label, icon } = input else {
				continue;
			};

            let custom_id = format!("{RANDOM_WINNER};{};{input_id}", self.user_id);
            let button = CreateButton::new(custom_id)
                .disabled(disabled)
                .emoji(icon.clone())
                .label(label)
                .style(ButtonStyle::Secondary);

            buttons.push(button);
        }

        Ok(buttons)
    }
}

impl AsRequest for Poll {
    fn as_req(&self) -> Req<Self> {
        Self::req(self.user_id)
    }
}

impl Request for Poll {
    type Args = UserId;

    fn req(user_id: Self::Args) -> Req<Self> {
        Req::new(NAME, user_id.to_string())
    }
}

#[async_trait]
impl ToEmbed for Poll {
    type Args = ();

    async fn to_embed(&self, ctx: &Context, _: Self::Args) -> Result<CreateEmbed> {
        let user = ctx.http.get_user(self.user_id).await?;

        let timestamp = timestamp_str(self.content.hours.get() * 60 * 60 * 1000, "R");
        let closes = format!("**Closes:** {timestamp}");
        let users = format!("**Users hidden:** {}", yes_no(self.content.hide_users));
        let output = format!("**Results hidden:** {}", yes_no(self.content.hide_output));
        let content = self.content.description.replace(['\r', '\n', '\t'], " ");

        let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
        let description = format!("{closes}\n{users}\n{output}\n\n> {content}");

        Ok(CreateEmbed::new()
            .author(author)
            .color(user.accent_colour.unwrap_or(BOT_COLOR))
            .description(description)
            .thumbnail(user.face())
            .title(&self.content.title))
    }
}

#[async_trait]
impl ToButtons for Poll {
    type Args = bool;

    async fn to_buttons(&self, ctx: &Context, disabled: Self::Args) -> Result<Vec<CreateButton>> {
        match self.content.kind {
            Kind::MultipleChoice => self.__to_buttons_multiple_choice(ctx, disabled).await,
            Kind::FreeResponse => self.__to_buttons_free_response(ctx, disabled).await,
            Kind::RandomWinner => self.__to_buttons_random_winner(ctx, disabled).await,
        }
    }
}

#[async_trait]
impl ToMessage for Poll {
    type Args = bool;

    async fn to_message(&self, ctx: &Context, disabled: Self::Args) -> Result<CreateMessage> {
        let mut message = CreateMessage::new().embed(self.to_embed(ctx, ()).await?);

        for button in self.to_buttons(ctx, disabled).await? {
            message = message.button(button);
        }

        Ok(message)
    }
}

#[async_trait]
impl ToModal for Poll {
    type Args = ();

    async fn to_modal(&self, _: &Context, _: Self::Args) -> Result<CreateModal> {
        self.ensure_kind(Kind::FreeResponse)?;

        let max = self.content.kind.max_inputs();
        let mut components = vec![];

        for input in self.inputs.iter().take(max) {
            let Input::FreeResponse { input_id, label, placeholder } = input else {
				continue;
			};

            let custom_id = input_id.to_string();
            let input = CreateInputText::new(InputTextStyle::Paragraph, label, custom_id)
                .placeholder(placeholder);

            components.push(CreateActionRow::InputText(input));
        }

        let custom_id = format!("{FREE_RESPONSE};{}", self.user_id);
        Ok(CreateModal::new(custom_id, "Submit a Response").components(components))
    }
}
