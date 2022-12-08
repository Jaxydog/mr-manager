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
    pub async fn send(&mut self, ctx: &Context, guild: GuildId, channel: ChannelId) -> Result<()> {
        let embed = self.try_as_embed(ctx, guild).await?;
        let mut builder = CreateMessage::new().embed(embed);

        for button in self.as_buttons(false, ()) {
            builder = builder.button(button);
        }

        if let Some(anchor) = self.anchor {
            if let Ok(message) = anchor.to_message(ctx).await {
                message.delete(ctx).await?;
            }
        }

        let message = channel.send_message(ctx, builder).await?;
        self.anchor = Some(Anchor::try_from(message)?);
        self.write().await
    }
}

impl Anchored for Config {
    fn anchor(&self) -> Result<Anchor> {
        self.anchor.ok_or(Error::MissingValue(Value::Anchor))
    }
}

impl NewReq for Config {
    type Args = GuildId;

    fn new_req(guild: Self::Args) -> Req<Self> {
        Req::new(format!("{NAME}/{guild}"), ".cfg")
    }
}

impl TryAsReq for Config {
    fn try_as_req(&self) -> Result<Req<Self>> {
        Ok(Self::new_req(self.anchor()?.guild))
    }
}

#[async_trait]
impl TryAsEmbedAsync for Config {
    type Args<'a> = GuildId;

    async fn try_as_embed(&self, ctx: &Context, guild: Self::Args<'_>) -> Result<CreateEmbed> {
        let guild = ctx.http.get_guild(guild).await?;
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

impl AsButtonVec for Config {
    type Args<'a> = ();

    fn as_buttons(&self, disabled: bool, _: Self::Args<'_>) -> Vec<CreateButton> {
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

impl AsModal for Config {
    type Args<'a> = ();

    fn as_modal(&self, _: Self::Args<'_>) -> CreateModal {
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
