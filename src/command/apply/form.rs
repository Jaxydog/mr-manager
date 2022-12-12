use rand::{thread_rng, Rng};

use super::*;

pub const OPTION_REASON: &str = "reason";

pub const BUTTON_ACCEPT: &str = formatcp!("{NAME}_accept");
pub const BUTTON_DENY: &str = formatcp!("{NAME}_deny");
pub const BUTTON_RESEND: &str = formatcp!("{NAME}_resend");

pub const MODAL_UPDATE: &str = formatcp!("{NAME}_update");

pub const TOAST: [&str; 8] = [
    "I spot a new member!",
    "A wild user appeared!",
    "Oh god, there's *another* one...",
    "This one seems... suspicious...",
    "Careful, they might bite!",
    "I'd keep an eye on this one.",
    "Aww, they didn't bring pizza!",
    "Who let *this* guy in..?",
];

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Pending = 0,
    Accepted = 1,
    Denied = 2,
    Resend = 3,
}

impl TryFrom<i64> for Status {
    type Error = Error;

    fn try_from(value: i64) -> Result<Self> {
        match value {
            0 => Ok(Self::Pending),
            1 => Ok(Self::Accepted),
            2 => Ok(Self::Denied),
            3 => Ok(Self::Resend),
            _ => Err(Error::InvalidValue(Value::Data, value.to_string())),
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let icon = match self {
            Self::Pending => '🤔',
            Self::Accepted => '👍',
            Self::Denied => '👎',
            Self::Resend => '🤷',
        };

        write!(f, "{icon} {self:?}")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Form {
    pub user: UserId,
    pub status: Status,
    pub reason: Option<String>,
    pub answers: Vec<String>,
    anchor: Option<Anchor>,
}

impl Form {
    pub fn new(user: UserId, answers: Vec<impl Into<String>>) -> Self {
        Self {
            user,
            status: Status::Pending,
            reason: None,
            answers: answers.into_iter().map(Into::into).collect(),
            anchor: None,
        }
    }
    pub fn created_string(&self) -> TimeString {
        let ms = self.anchor().map_or_else(
            |_| Utc::now().timestamp_millis(),
            |a| a.message.created_at().timestamp_millis(),
        );

        TimeString::new(ms).flag(TimeFlag::Relative)
    }
    pub async fn send(&mut self, http: &Http, guild: GuildId, channel: ChannelId) -> Result<()> {
        let mut builder = CreateMessage::new().embed(self.as_embed(http, guild).await?);

        for button in self.as_buttons(self.status != Status::Pending, ()) {
            builder = builder.button(button);
        }

        let message = channel.send_message(http, builder).await?;

        self.anchor = Some(Anchor::try_from((guild, message))?);
        self.try_write(())
    }
    #[allow(clippy::match_same_arms, clippy::match_wildcard_for_single_variants)] // prevents false positives, intended
    pub async fn notify(&self, http: &Http, guild: GuildId) -> Result<()> {
        let guild = http.get_guild(guild).await?;

        let Ok(channel) = self.user.create_dm_channel(http).await else {
            return Ok(())
        };

        let mut author = CreateEmbedAuthor::new(&guild.name);

        if let Some(icon_url) = guild.icon_url() {
            author = author.icon_url(icon_url);
        }

        let mut description = match self.status {
            Status::Accepted => include_str!(r"..\..\include\apply\accept.txt"),
            Status::Denied => include_str!(r"..\..\include\apply\deny.txt"),
            Status::Resend => include_str!(r"..\..\include\apply\resend.txt"),
            s => return Err(Error::InvalidValue(Value::Other("Status"), s.to_string())),
        }
        .to_string();

        if let Some(reason) = self.reason.as_ref() {
            description.push_str(&format!("\n\n> {reason}"));
        }

        let embed = CreateEmbed::new()
            .author(author)
            .color(BOT_COLOR)
            .description(description)
            .title(match self.status {
                Status::Accepted => "Your application has been accepted!",
                Status::Denied => "Your application has been denied.",
                Status::Resend => "You have been asked to resubmit your application.",
                s => return Err(Error::InvalidValue(Value::Other("Status"), s.to_string())),
            });

        channel
            .send_message(http, CreateMessage::new().embed(embed))
            .await?;

        Ok(())
    }
    pub async fn update(
        &mut self,
        http: &Http,
        guild: GuildId,
        role: RoleId,
        status: Status,
        reason: Option<impl Send + Sync + Into<String>>,
    ) -> Result<()> {
        let mut member = guild.member(http, self.user).await?;

        self.status = status;
        self.reason = reason.map(Into::into);
        self.try_write(())?;

        if status == Status::Accepted {
            member.add_role(http, role).await?;
        } else {
            member.remove_role(http, role).await?;
        }

        if let Some(anchor) = self.anchor {
            let mut message = anchor.to_message(http).await?;
            let mut builder = EditMessage::new().embed(self.as_embed(http, guild).await?);

            for button in self.as_buttons(status != Status::Pending, ()) {
                builder = builder.button(button);
            }

            message.edit(http, builder).await?;
        }

        self.notify(http, guild).await
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

impl TryAsReq<()> for Form {
    fn try_as_req(&self, _: ()) -> Result<Req<Self>> {
        Ok(Self::new_req((self.anchor()?.guild, self.user)))
    }
}

impl AsReq<GuildId> for Form {
    fn as_req(&self, guild: GuildId) -> Req<Self> {
        Self::new_req((guild, self.user))
    }
}

#[async_trait]
impl AsEmbedAsync<GuildId> for Form {
    async fn as_embed(&self, http: &Http, guild: GuildId) -> Result<CreateEmbed> {
        let config = Config::read(guild)?;
        let user = http.get_user(self.user).await?;

        let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
        let color = user.accent_colour.unwrap_or(BOT_COLOR);
        let title = TOAST[thread_rng().gen_range(0..TOAST.len())];
        let mut description = format!("**Profile:** <@{}>\n", self.user);

        description.push_str(&format!("**Received:** {}\n", self.created_string()));
        description.push_str(&format!("**Status:** {}\n", self.status));

        if let Some(reason) = self.reason.as_ref() {
            description.push_str(&format!("**Reason:** {reason}"));
        }

        let mut embed = CreateEmbed::new()
            .author(author)
            .color(color)
            .description(description)
            .thumbnail(user.face())
            .title(title);

        for (index, question) in config.content.questions.iter().enumerate() {
            let answer = self
                .answers
                .get(index)
                .map_or_else(|| "N/A".to_string(), |s| format!("> {s}"));

            embed = embed.field(question, answer, false);
        }

        Ok(embed)
    }
}

impl AsButtonVec<()> for Form {
    fn as_buttons(&self, disabled: bool, _: ()) -> Vec<CreateButton> {
        let accept = CreateButton::new(CustomId::new(BUTTON_ACCEPT).arg(self.user.to_string()))
            .disabled(disabled)
            .emoji('👍')
            .label("Accept")
            .style(ButtonStyle::Success);
        let deny = CreateButton::new(CustomId::new(BUTTON_DENY).arg(self.user.to_string()))
            .disabled(disabled)
            .emoji('👎')
            .label("Deny")
            .style(ButtonStyle::Danger);
        let resend = CreateButton::new(CustomId::new(BUTTON_RESEND).arg(self.user.to_string()))
            .disabled(disabled)
            .emoji('🤷')
            .label("Resend")
            .style(ButtonStyle::Secondary);

        vec![accept, deny, resend]
    }
}

impl AsModal<Status> for Form {
    fn as_modal(&self, status: Status) -> CreateModal {
        let custom_id = CustomId::new(MODAL_UPDATE)
            .arg(self.user.to_string())
            .arg((status as u8).to_string());

        let modal = CreateModal::new(custom_id, "Update Application");
        let components = vec![CreateActionRow::InputText(
            CreateInputText::new(InputTextStyle::Short, "Reason (optional)", OPTION_REASON)
                .max_length(256)
                .required(false),
        )];

        modal.components(components)
    }
}
