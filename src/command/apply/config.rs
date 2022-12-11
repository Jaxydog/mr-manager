use super::*;

pub const CM_MODAL: &str = formatcp!("{NAME}_modal");
pub const CM_ABOUT: &str = formatcp!("{NAME}_about");

pub const MD_SUBMIT: &str = formatcp!("{NAME}_submit");

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Content {
    pub title: String,
    pub description: String,
    pub thumbnail: String,
    pub questions: Vec<String>,
}

impl Content {
    pub fn new(
        title: impl Into<String>,
        description: impl Into<String>,
        thumbnail: impl Into<String>,
        questions: Vec<impl Into<String>>,
    ) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            thumbnail: thumbnail.into(),
            questions: questions.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub channel: ChannelId,
    pub role: RoleId,
    pub content: Content,
    anchor: Option<Anchor>,
}

impl Config {
    pub const fn new(channel: ChannelId, role: RoleId, content: Content) -> Self {
        Self {
            channel,
            role,
            content,
            anchor: None,
        }
    }
    pub async fn send(&mut self, http: &Http, guild: GuildId, channel: ChannelId) -> Result<()> {
        let embed = self.as_embed(http, guild).await?;
        let mut builder = CreateMessage::new().embed(embed);

        for button in self.as_buttons(false, ()) {
            builder = builder.button(button);
        }

        if let Some(anchor) = self.anchor {
            if let Ok(message) = anchor.to_message(http).await {
                message.delete(http).await?;
            }
        }

        let message = channel.send_message(http, builder).await?;
        self.anchor = Some(Anchor::try_from(message)?);
        self.try_write(())
    }
}

impl Anchored for Config {
    fn anchor(&self) -> Result<Anchor> {
        self.anchor.ok_or(Error::MissingValue(Value::Anchor))
    }
}

impl NewReq<GuildId> for Config {
    fn new_req(guild: GuildId) -> Req<Self> {
        Req::new(format!("{NAME}/{guild}"), ".cfg")
    }
}

impl TryAsReq<()> for Config {
    fn try_as_req(&self, _: ()) -> Result<Req<Self>> {
        Ok(Self::new_req(self.anchor()?.guild))
    }
}

impl AsReq<GuildId> for Config {
    fn as_req(&self, guild: GuildId) -> Req<Self> {
        Self::new_req(guild)
    }
}

#[async_trait]
impl AsEmbedAsync<GuildId> for Config {
    async fn as_embed(&self, http: &Http, guild: GuildId) -> Result<CreateEmbed> {
        let guild = http.get_guild(guild).await?;
        let footer = CreateEmbedFooter::new(format!("Questions: {}", self.content.questions.len()));
        let mut author = CreateEmbedAuthor::new(&guild.name);

        if let Some(icon_url) = guild.icon_url() {
            author = author.icon_url(icon_url);
        }

        Ok(CreateEmbed::new()
            .author(author)
            .color(BOT_COLOR)
            .description(&self.content.description)
            .footer(footer)
            .thumbnail(&self.content.thumbnail)
            .title(&self.content.title))
    }
}

impl AsButtonVec<()> for Config {
    fn as_buttons(&self, disabled: bool, _: ()) -> Vec<CreateButton> {
        let modal = CreateButton::new(CM_MODAL)
            .disabled(disabled)
            .emoji('👋')
            .label("Apply to Guild")
            .style(ButtonStyle::Primary);
        let about = CreateButton::new(CM_ABOUT)
            .disabled(disabled)
            .emoji('ℹ')
            .label("About Applications")
            .style(ButtonStyle::Secondary);

        vec![modal, about]
    }
}

impl AsModal<()> for Config {
    fn as_modal(&self, _: ()) -> CreateModal {
        let modal = CreateModal::new(MD_SUBMIT, "Apply to Guild");
        let mut components = vec![];

        for (index, question) in self.content.questions.iter().enumerate() {
            let custom_id = index.to_string();
            let input = CreateInputText::new(InputTextStyle::Paragraph, question, custom_id);
            let row = CreateActionRow::InputText(input.max_length(1024).required(true));

            components.push(row);
        }

        modal.components(components)
    }
}
