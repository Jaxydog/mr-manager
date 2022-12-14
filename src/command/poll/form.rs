use std::collections::{BTreeMap, BTreeSet};

use super::*;

pub const BUTTON_REMOVE: &str = formatcp!("{NAME}_remove");
pub const BUTTON_RESULTS: &str = formatcp!("{NAME}_results");

pub const MODAL_SUBMIT: &str = formatcp!("{NAME}_submit");

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Choice,
    Response,
    Raffle,
}

impl TryFrom<i64> for Kind {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self> {
        match value {
            0 => Ok(Self::Choice),
            1 => Ok(Self::Response),
            2 => Ok(Self::Raffle),
            _ => Err(Error::InvalidId(Value::Data, value.to_string())),
        }
    }
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let emoji = match self {
            Self::Choice => '🔢',
            Self::Response => '📝',
            Self::Raffle => '🎲',
        };

        write!(f, "{emoji} {self:?}")
    }
}

#[repr(transparent)]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Active(pub BTreeSet<(GuildId, UserId)>);

impl NewReq<()> for Active {
    fn new_req(_: ()) -> Req<Self> {
        Req::new(NAME, ".dat")
    }
}

impl AsReq<()> for Active {
    fn as_req(&self, _: ()) -> Req<Self> {
        Self::new_req(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Content {
    pub title: String,
    pub description: String,
    pub hours: i64,
    pub image: Option<String>,
    pub hide_members: bool,
    pub hide_results: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Reply {
    Choice(usize),
    Response(Vec<String>),
    Raffle,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Form {
    pub user: UserId,
    pub kind: Kind,
    pub content: Content,
    pub inputs: Vec<Input>,
    pub replies: BTreeMap<UserId, Reply>,
    anchor: Option<Anchor>,
    output: Option<Output>,
}

impl Form {
    pub const fn new(user: UserId, kind: Kind, content: Content) -> Self {
        Self {
            user,
            kind,
            content,
            inputs: vec![],
            replies: BTreeMap::new(),
            anchor: None,
            output: None,
        }
    }

    pub fn output(&mut self) -> &Output {
        let cloned = self.clone();

        self.output.get_or_insert_with(|| Output::new(&cloned))
    }
    pub fn closes_at(&self) -> TimeString {
        let ms = self.content.hours * 60 * 60 * 1000;
        let base = self.anchor().map_or_else(
            |_| Utc::now().timestamp_millis(),
            |a| a.message.created_at().timestamp_millis(),
        );

        TimeString::new(base + ms)
    }

    pub fn as_remove_buttons(&self, disabled: bool) -> Vec<CreateButton> {
        let mut buttons = vec![];

        for (index, input) in self.inputs.iter().enumerate() {
            let label = match input {
                Input::Choice(data) => &data.label,
                Input::Response(data) => &data.label,
            };

            let custom_id = CustomId::new(BUTTON_REMOVE).arg(self.user).arg(index);
            let button = CreateButton::new(custom_id)
                .disabled(disabled)
                .label(label)
                .style(ButtonStyle::Danger);

            buttons.push(button);
        }

        buttons
    }
    #[allow(clippy::unused_self)]
    pub fn as_remove_embed(&self) -> CreateEmbed {
        CreateEmbed::new().color(BOT_COLOR).title("Remove Inputs")
    }
    pub fn as_remove_message(&self, disabled: bool) -> CreateInteractionResponseMessage {
        let mut builder = CreateInteractionResponseMessage::new().embed(self.as_remove_embed());

        for button in self.as_remove_buttons(disabled) {
            builder = builder.button(button);
        }

        builder.ephemeral(true)
    }

    pub fn as_results_buttons(&self, disabled: bool, message: MessageId) -> Vec<CreateButton> {
        vec![
            CreateButton::new(CustomId::new(BUTTON_RESULTS).arg(self.user).arg(message))
                .disabled(disabled)
                .emoji('📊')
                .label("View Results")
                .style(ButtonStyle::Primary),
        ]
    }
    pub async fn as_results_embed(&self, http: &Http) -> Result<CreateEmbed> {
        let user = http.get_user(self.user).await?;
        let color = user.accent_colour.unwrap_or(BOT_COLOR);
        let mut builder = CreateEmbed::new().color(color).title("Poll Results");

        if let Ok(anchor) = self.anchor() {
            builder = builder.url(anchor.to_string());
        }
        if self.content.hide_results {
            let footer = CreateEmbedFooter::new("Results are only visible to the poll author!");

            builder = builder.footer(footer);
        }

        Ok(builder)
    }
    pub async fn as_results_message(&self, http: &Http, disabled: bool) -> Result<CreateMessage> {
        let message = self.anchor()?.message;
        let mut builder = CreateMessage::new().embed(self.as_results_embed(http).await?);

        for button in self.as_results_buttons(disabled, message) {
            builder = builder.button(button);
        }

        Ok(builder)
    }

    pub async fn send(
        &mut self,
        http: &Http,
        guild: GuildId,
        channel: ChannelId,
        force: bool,
    ) -> Result<()> {
        if self.is_anchored() && !force {
            return Err(Error::Other("The poll has already been sent"));
        }

        if let Ok(anchor) = self.anchor() {
            anchor.to_message(http).await?.delete(http).await?;
        }

        let guild = http.get_guild(guild).await?;
        let Some(channel) = guild.channels(http).await?.remove(&channel) else {
            return Err(Error::InvalidId(Value::Channel, channel.to_string()));
        };

        let builder = self.as_message(http, false).await?;
        let message = channel.send_message(http, builder).await?;
        self.anchor = Some(Anchor::try_from((guild.id, &message))?);
        self.write(self.anchor()?.guild)?;

        let mut active = Active::read(()).unwrap_or_default();
        active.0.insert((self.anchor()?.guild, self.user));
        active.write(())
    }
    pub async fn close(mut self, http: &Http) -> Result<()> {
        let Ok(anchor) = self.anchor() else {
            return Err(Error::Other("The poll has not been sent"));
        };

        let mut active = Active::read(()).unwrap_or_default();
        active.0.remove(&(anchor.guild, self.user));
        active.write(())?;

        let mut message = anchor.to_message(http).await?;
        let mut builder = EditMessage::new().components(vec![]);

        for button in self.as_buttons(true, ()) {
            builder = builder.button(button);
        }

        message.edit(http, builder).await?;

        let builder = self.as_results_message(http, false).await?;
        anchor.channel.send_message(http, builder).await?;

        self.output = Some(Output::new(&self));
        self.write((anchor.guild, anchor.message))?;
        self.remove(anchor.guild)
    }

    fn __as_buttons_choice(&self, disabled: bool) -> Vec<CreateButton> {
        let count = Input::max_count(Kind::Choice);
        let mut buttons = vec![];

        for (index, input) in self.inputs.iter().enumerate().take(count) {
            let Input::Choice(data) = input else {
				continue;
			};

            buttons.push(data.as_button(disabled, (self.user, index)));
        }

        buttons
    }
    fn __as_buttons_response(&self, disabled: bool) -> Vec<CreateButton> {
        vec![
            CreateButton::new(CustomId::new(BUTTON_RESPONSE).arg(self.user))
                .disabled(disabled)
                .emoji('📩')
                .label("Submit Response")
                .style(ButtonStyle::Primary),
        ]
    }
    fn __as_buttons_raffle(&self, disabled: bool) -> Vec<CreateButton> {
        vec![
            CreateButton::new(CustomId::new(BUTTON_RAFFLE).arg(self.user))
                .disabled(disabled)
                .emoji('🎲')
                .label("Enter Raffle")
                .style(ButtonStyle::Primary),
        ]
    }
}

impl Anchored for Form {
    fn anchor(&self) -> Result<Anchor> {
        self.anchor.ok_or(Error::MissingValue(Value::Anchor))
    }
}

impl NewReq<(GuildId, UserId)> for Form {
    fn new_req((guild, user): (GuildId, UserId)) -> Req<Self> {
        Req::new(format!("{NAME}/{guild}"), user)
    }
}

impl NewReq<(GuildId, UserId, MessageId)> for Form {
    fn new_req((guild, user, message): (GuildId, UserId, MessageId)) -> Req<Self> {
        Req::new(format!("{NAME}/{guild}/{user}"), message)
    }
}

impl AsReq<GuildId> for Form {
    fn as_req(&self, guild: GuildId) -> Req<Self> {
        Self::new_req((guild, self.user))
    }
}

impl AsReq<(GuildId, MessageId)> for Form {
    fn as_req(&self, (guild, message): (GuildId, MessageId)) -> Req<Self> {
        Self::new_req((guild, self.user, message))
    }
}

impl AsButtonVec<()> for Form {
    fn as_buttons(&self, disabled: bool, _: ()) -> Vec<CreateButton> {
        match self.kind {
            Kind::Choice => self.__as_buttons_choice(disabled),
            Kind::Response => self.__as_buttons_response(disabled),
            Kind::Raffle => self.__as_buttons_raffle(disabled),
        }
    }
}

#[async_trait]
impl AsEmbedAsync<()> for Form {
    async fn as_embed(&self, http: &Http, _: ()) -> Result<CreateEmbed> {
        let user = http.get_user(self.user).await?;

        let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());

        let members = if self.content.hide_members {
            "*Members are hidden*"
        } else {
            "*Members are shown*"
        };

        let results = if self.content.hide_results {
            "*Results are hidden*"
        } else {
            "*Results are shown*"
        };

        let mut description = format!("**Type:** {}\n", self.kind);
        description.push_str(&format!("**Closes:** {}\n\n", self.closes_at()));
        description.push_str(&format!("{members}\n{results}\n\n> ",));
        description.push_str(&self.content.description);

        let mut builder = CreateEmbed::new()
            .author(author)
            .color(user.accent_colour.unwrap_or(BOT_COLOR))
            .description(description)
            .thumbnail(user.face())
            .title(&self.content.title);

        if let Some(url) = self.content.image.as_ref() {
            builder = builder.image(url);
        }

        Ok(builder)
    }
}

#[async_trait]
impl AsMessageAsync<bool> for Form {
    async fn as_message(&self, http: &Http, disabled: bool) -> Result<CreateMessage> {
        let mut builder = CreateMessage::new().embed(self.as_embed(http, ()).await?);

        for button in self.as_buttons(disabled, ()) {
            builder = builder.button(button);
        }

        Ok(builder)
    }
}

impl TryAsModal<()> for Form {
    fn try_as_modal(&self, _: ()) -> Result<CreateModal> {
        if self.kind != Kind::Response {
            return Err(Error::InvalidValue(Value::Data, self.kind.to_string()));
        }
        if self.inputs.is_empty() {
            return Err(Error::Other("You must provide at least one input"));
        }

        let count = Input::max_count(Kind::Response);
        let custom_id = CustomId::new(MODAL_SUBMIT).arg(self.user);
        let mut components = vec![];

        for (index, input) in self.inputs.iter().enumerate().take(count) {
            let Input::Response(data) = input else {
				continue;
			};

            components.push(CreateActionRow::InputText(data.as_input_text(index)));
        }

        Ok(CreateModal::new(custom_id, "Submit Response").components(components))
    }
}
