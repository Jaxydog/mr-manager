use rand::{thread_rng, Rng};

use crate::prelude::*;

pub const NAME: &str = "apply";

pub const CM_MODAL: &str = formatcp!("{NAME}_modal");
pub const CM_ABOUT: &str = formatcp!("{NAME}_about");
pub const CM_ACCEPT: &str = formatcp!("{NAME}_accept");
pub const CM_DENY: &str = formatcp!("{NAME}_deny");
pub const CM_RESEND: &str = formatcp!("{NAME}_resend");

pub const MD_SUBMIT: &str = formatcp!("{NAME}_submit");
pub const MD_UPDATE: &str = formatcp!("{NAME}_update");

pub const SC_SETUP: &str = "setup";
pub const SC_EDIT: &str = "edit";
pub const SC_UPDATE: &str = "update";
pub const SC_REMOVE: &str = "remove";

pub const OP_TITLE: &str = "title";
pub const OP_DESCRIPTION: &str = "content";
pub const OP_THUMBNAIL: &str = "thumbnail_link";
pub const OP_CHANNEL: &str = "output_channel";
pub const OP_QUESTION_1: &str = "question_1";
pub const OP_QUESTION_2: &str = "question_2";
pub const OP_QUESTION_3: &str = "question_3";
pub const OP_QUESTION_4: &str = "question_4";
pub const OP_QUESTION_5: &str = "question_5";
pub const OP_ROLE: &str = "accept_role";
pub const OP_USER: &str = "user";
pub const OP_STATUS: &str = "status";
pub const OP_REASON: &str = "reason";

pub const TOAST: [&str; 8] = [
    "I spot a new member!",
    "A wild user appeared!",
    "Oh god, there's *another* one.",
    "This one seems... *suspicious...*",
    "Careful, they might bite!",
    "I'd keep an eye on this one...",
    "They didn't bring pizza.",
    "Who let *this* guy in?",
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Content {
    title: String,
    description: String,
    thumbnail: String,
    questions: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    channel: ChannelId,
    role: RoleId,
    content: Content,
    anchor: Option<Anchor>,
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
impl ToEmbedAsync for Config {
    type Args = GuildId;

    async fn to_embed(&self, ctx: &Context, guild: Self::Args) -> Result<CreateEmbed> {
        let guild = guild.to_partial_guild(ctx).await?;
        let description = self.content.description.replace(['\r', '\n'], "\n");
        let footer = CreateEmbedFooter::new(format!("Questions: {}", self.content.questions.len()));
        let mut author = CreateEmbedAuthor::new(&guild.name);

        if let Some(icon_url) = guild.icon_url() {
            author = author.icon_url(icon_url);
        }

        Ok(CreateEmbed::new()
            .author(author)
            .color(BOT_COLOR)
            .description(description.trim())
            .footer(footer)
            .thumbnail(self.content.thumbnail.as_str())
            .title(self.content.title.trim()))
    }
}

impl ToButtonArray for Config {
    type Args = bool;

    fn to_button_array(&self, disabled: Self::Args) -> Result<Vec<CreateButton>> {
        Ok(vec![
            CreateButton::new(CM_MODAL)
                .disabled(disabled)
                .emoji('👋')
                .label("Apply to Guild")
                .style(ButtonStyle::Primary),
            CreateButton::new(CM_ABOUT)
                .disabled(disabled)
                .emoji('ℹ')
                .label("About Applications")
                .style(ButtonStyle::Secondary),
        ])
    }
}

impl ToModal for Config {
    type Args = ();

    fn to_modal(&self, _: Self::Args) -> Result<CreateModal> {
        let modal = CreateModal::new(MD_SUBMIT, "Apply to Guild");
        let mut components = vec![];

        for (index, question) in self.content.questions.iter().enumerate() {
            let custom_id = index.to_string();
            let input = CreateInputText::new(InputTextStyle::Paragraph, question, custom_id);

            components.push(CreateActionRow::InputText(
                input.max_length(1024).required(true),
            ));
        }

        Ok(modal.components(components))
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Pending,
    Accepted,
    Denied,
    Resend,
}

impl From<Status> for i32 {
    fn from(s: Status) -> Self {
        s as i32
    }
}

impl TryFrom<i32> for Status {
    type Error = Error;

    fn try_from(value: i32) -> Result<Self> {
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
    user: UserId,
    status: Status,
    reason: Option<String>,
    answers: Vec<(usize, String)>,
    anchor: Option<Anchor>,
}

impl Form {
    pub async fn update(
        &mut self,
        ctx: &Context,
        status: Status,
        reason: Option<impl Send + Sync + Into<String>>,
    ) -> Result<()> {
        self.status = status;
        self.reason = reason.map(Into::into);
        self.write().await?;

        if let Some(anchor) = self.anchor {
            let mut message = anchor.to_message(ctx).await?;
            let mut edit = EditMessage::new().embed(self.to_embed(ctx, anchor.guild).await?);

            for button in self.to_button_array(status != Status::Pending)? {
                edit = edit.button(button);
            }

            message.edit(ctx, edit).await?;
        }

        Ok(())
    }
}

impl Anchored for Form {
    fn anchor(&self) -> Result<Anchor> {
        self.anchor.ok_or(Error::MissingValue(Value::Anchor))
    }
}

impl NewReq for Form {
    type Args = (GuildId, UserId);

    fn new_req((guild, user): Self::Args) -> Req<Self> {
        Req::new(format!("{NAME}/{guild}"), user.to_string())
    }
}

impl TryAsReq for Form {
    fn try_as_req(&self) -> Result<Req<Self>> {
        Ok(Self::new_req((self.anchor()?.guild, self.user)))
    }
}

#[async_trait]
impl ToEmbedAsync for Form {
    type Args = GuildId;

    async fn to_embed(&self, ctx: &Context, guild: Self::Args) -> Result<CreateEmbed> {
        let config = Config::read(guild).await?;
        let user = ctx.http.get_user(self.user).await?;

        let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
        let color = user.accent_colour.unwrap_or(BOT_COLOR);
        let title = TOAST[thread_rng().gen_range(0..TOAST.len())];
        let mut description = String::new();

        description.push_str(&format!("**Profile:** <@{}>\n", self.user));

        let time = TimeString::new(self.anchor.map_or_else(
            || Utc::now().timestamp_millis(),
            |anchor| anchor.message.created_at().timestamp_millis(),
        ))
        .flag(TimeFlag::Relative);

        description.push_str(&format!("**Received:** {time}\n"));
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
                .iter()
                .find_map(|(n, s)| (*n == index).then_some(format!("> {s}")))
                .unwrap_or_else(|| "N/A".to_string());

            embed = embed.field(question, answer, false);
        }

        Ok(embed)
    }
}

impl ToButtonArray for Form {
    type Args = bool;

    fn to_button_array(&self, disabled: Self::Args) -> Result<Vec<CreateButton>> {
        let accept = CustomId::new(CM_ACCEPT).arg(self.user.to_string());
        let deny = CustomId::new(CM_DENY).arg(self.user.to_string());
        let resend = CustomId::new(CM_RESEND).arg(self.user.to_string());

        Ok(vec![
            CreateButton::new(accept.to_string())
                .disabled(disabled)
                .emoji('👍')
                .label("Accept")
                .style(ButtonStyle::Success),
            CreateButton::new(deny.to_string())
                .disabled(disabled)
                .emoji('👎')
                .label("Deny")
                .style(ButtonStyle::Danger),
            CreateButton::new(resend.to_string())
                .disabled(disabled)
                .emoji('🤷')
                .label("Request Resend")
                .style(ButtonStyle::Secondary),
        ])
    }
}

impl ToModal for Form {
    type Args = Status;

    fn to_modal(&self, status: Self::Args) -> Result<CreateModal> {
        let custom_id = CustomId::new(MD_UPDATE)
            .arg(self.user.to_string())
            .arg(i32::from(status).to_string());

        let modal = CreateModal::new(custom_id.to_string(), "Update Application");
        let components = vec![CreateActionRow::InputText(
            CreateInputText::new(InputTextStyle::Short, "Reason (Optional)", OP_REASON)
                .max_length(256)
                .required(false),
        )];

        Ok(modal.components(components))
    }
}

#[allow(clippy::too_many_lines)]
pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .default_member_permissions(Permissions::MODERATE_MEMBERS)
        .description("Manage guild applications")
        .dm_permission(false)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                SC_SETUP,
                "Set up guild applications",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OP_TITLE,
                    "The title of the entry embed",
                )
                .max_length(256)
                .clone()
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OP_DESCRIPTION,
                    "The description of the entry embed",
                )
                .max_length(4096)
                .clone()
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OP_THUMBNAIL,
                    "The thumbnail of the entry embed",
                )
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    OP_CHANNEL,
                    "The channel that completed forms are sent to",
                )
                .channel_types(vec![ChannelType::Text])
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Role,
                    OP_ROLE,
                    "The role that is given to accepted members",
                )
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OP_QUESTION_1,
                    "The first question of the application",
                )
                .max_length(45)
                .clone()
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OP_QUESTION_2,
                    "The second question of the application",
                )
                .max_length(45)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OP_QUESTION_3,
                    "The third question of the application",
                )
                .max_length(45)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OP_QUESTION_4,
                    "The fourth question of the application",
                )
                .max_length(45)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OP_QUESTION_5,
                    "The fifth question of the application",
                )
                .max_length(45)
                .clone(),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                SC_EDIT,
                "Edit the guild application configuration",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OP_TITLE,
                    "The title of the entry embed",
                )
                .max_length(256)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OP_DESCRIPTION,
                    "The description of the entry embed",
                )
                .max_length(4096)
                .clone(),
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::String,
                OP_THUMBNAIL,
                "The thumbnail of the entry embed",
            ))
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    OP_CHANNEL,
                    "The channel that completed forms are sent to",
                )
                .channel_types(vec![ChannelType::Text]),
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::Role,
                OP_ROLE,
                "The role that is given to accepted members",
            ))
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OP_QUESTION_1,
                    "The first question of the application",
                )
                .max_length(45)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OP_QUESTION_2,
                    "The second question of the application",
                )
                .max_length(45)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OP_QUESTION_3,
                    "The third question of the application",
                )
                .max_length(45)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OP_QUESTION_4,
                    "The fourth question of the application",
                )
                .max_length(45)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OP_QUESTION_5,
                    "The fifth question of the application",
                )
                .max_length(45)
                .clone(),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                SC_UPDATE,
                "Update a guild application",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::User,
                    OP_USER,
                    "The user who submitted the application",
                )
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    OP_STATUS,
                    "The target status of the application",
                )
                .add_int_choice(Status::Accepted.to_string(), i32::from(Status::Accepted))
                .add_int_choice(Status::Denied.to_string(), i32::from(Status::Denied))
                .add_int_choice(Status::Resend.to_string(), i32::from(Status::Resend))
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OP_REASON,
                    "The provided reason",
                )
                .max_length(256)
                .clone(),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                SC_REMOVE,
                "Remove a guild application",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::User,
                    OP_USER,
                    "The user who submitted the application",
                )
                .required(true),
            ),
        )
}

#[allow(clippy::too_many_lines)]
pub async fn run_command(ctx: &Context, cmd: &CommandInteraction) -> Result<()> {
    let Some(guild) = cmd.guild_id else {
        return Err(Error::MissingId(Value::Guild));
    };

    let o = &cmd.data.options();

    if let Ok(o) = get_subcommand(o, SC_SETUP) {
        let mut config = Config {
            channel: get_channel(o, OP_CHANNEL)?.id,
            role: get_role(o, OP_ROLE)?.id,
            content: Content {
                title: get_str(o, OP_TITLE)?.to_string(),
                description: get_str(o, OP_DESCRIPTION)?
                    .replace(r"\n", "\n")
                    .trim()
                    .to_string(),
                thumbnail: get_str(o, OP_THUMBNAIL)?.to_string(),
                questions: [
                    get_str(o, OP_QUESTION_1),
                    get_str(o, OP_QUESTION_2),
                    get_str(o, OP_QUESTION_3),
                    get_str(o, OP_QUESTION_4),
                    get_str(o, OP_QUESTION_5),
                ]
                .into_iter()
                .filter_map(Result::ok)
                .map(ToString::to_string)
                .collect(),
            },
            anchor: None,
        };

        let mut message = CreateMessage::new().embed(config.to_embed(ctx, guild).await?);

        for button in config.to_button_array(false)? {
            message = message.button(button);
        }

        let message = cmd.channel_id.send_message(ctx, message).await?;
        config.anchor = Some(Anchor::try_from(message)?);
        config.write().await?;

        let embed = CreateEmbed::new()
            .color(BOT_COLOR)
            .title("Set up applications!");
        let message = CreateInteractionResponseMessage::new()
            .embed(embed)
            .ephemeral(true);

        cmd.create_response(ctx, CreateInteractionResponse::Message(message))
            .await
            .map_err(Error::from)
    } else if let Ok(o) = get_subcommand(o, SC_EDIT) {
        let mut config = Config::read(guild).await?;
        let mut update = false;

        if let Ok(title) = get_str(o, OP_TITLE) {
            config.content.title = title.to_string();
            update = true;
        }
        if let Ok(description) = get_str(o, OP_DESCRIPTION) {
            config.content.description = description.replace(r"\n", "\n").trim().to_string();
            update = true;
        }
        if let Ok(thumbnail) = get_str(o, OP_THUMBNAIL) {
            config.content.thumbnail = thumbnail.to_string();
            update = true;
        }
        if let Ok(channel) = get_channel(o, OP_CHANNEL) {
            config.channel = channel.id;
        }
        if let Ok(role) = get_role(o, OP_ROLE) {
            config.role = role.id;
        }
        if let Ok(question) = get_str(o, OP_QUESTION_1) {
            config.content.questions[0] = question.to_string();
            update = true;
        }
        if let Ok(question) = get_str(o, OP_QUESTION_2) {
            config.content.questions[1] = question.to_string();
            update = true;
        }
        if let Ok(question) = get_str(o, OP_QUESTION_3) {
            config.content.questions[2] = question.to_string();
            update = true;
        }
        if let Ok(question) = get_str(o, OP_QUESTION_4) {
            config.content.questions[3] = question.to_string();
            update = true;
        }
        if let Ok(question) = get_str(o, OP_QUESTION_5) {
            config.content.questions[4] = question.to_string();
            update = true;
        }

        config.write().await?;

        if update {
            let mut message = config.anchor()?.to_message(ctx).await?;
            let edit = EditMessage::new().embed(config.to_embed(ctx, guild).await?);

            message.edit(ctx, edit).await?;
        }

        let embed = CreateEmbed::new()
            .color(BOT_COLOR)
            .title("Edited configuration!");
        let message = CreateInteractionResponseMessage::new()
            .embed(embed)
            .ephemeral(true);

        cmd.create_response(ctx, CreateInteractionResponse::Message(message))
            .await
            .map_err(Error::from)
    } else if let Ok(o) = get_subcommand(o, SC_UPDATE) {
        let (user, _) = get_user(o, OP_USER)?;
        let mut form = Form::read((guild, user.id)).await?;

        let status = get_i64(o, OP_STATUS)?;
        let Ok(status) = i32::try_from(status) else {
            return Err(Error::InvalidValue(Value::Other("Status"), status.to_string()))
        };
        let status = Status::try_from(status)?;

        form.update(ctx, status, get_str(o, OP_REASON).ok()).await?;

        let embed = CreateEmbed::new()
            .color(BOT_COLOR)
            .title("Updated application!");
        let message = CreateInteractionResponseMessage::new()
            .embed(embed)
            .ephemeral(true);

        cmd.create_response(ctx, CreateInteractionResponse::Message(message))
            .await
            .map_err(Error::from)
    } else if let Ok(o) = get_subcommand(o, SC_REMOVE) {
        let config = Config::read(guild).await?;
        let (user, _) = get_user(o, OP_USER)?;

        let form = Form::read((guild, user.id)).await?;
        let mut member = guild.member(ctx, user.id).await?;

        if let Some(anchor) = form.anchor.as_ref() {
            let message = anchor.to_message(ctx).await?;

            message.delete(ctx).await?;
        }

        form.remove().await?;
        member.remove_role(ctx, config.role).await?;

        let embed = CreateEmbed::new()
            .color(BOT_COLOR)
            .title("Removed application!");
        let message = CreateInteractionResponseMessage::new()
            .embed(embed)
            .ephemeral(true);

        cmd.create_response(ctx, CreateInteractionResponse::Message(message))
            .await
            .map_err(Error::from)
    } else {
        Err(Error::InvalidId(Value::Command, cmd.data.name.clone()))
    }
}
pub async fn run_component(ctx: &Context, cpn: &mut ComponentInteraction) -> Result<()> {
    let custom_id = CustomId::try_from(cpn.data.custom_id.as_str())?;
    let Some(guild) = cpn.guild_id else {
        return Err(Error::MissingId(Value::Guild));
    };

    match custom_id.name.as_str() {
        CM_MODAL => {
            if let Ok(form) = Form::read((guild, cpn.user.id)).await {
                match form.status {
                    Status::Pending => {
                        return Err(Error::Other("Your application is currently pending"))
                    }
                    Status::Accepted => {
                        return Err(Error::Other("Your application has already been accepted"))
                    }
                    Status::Denied => {
                        return Err(Error::Other("Your application has already been denied"))
                    }
                    Status::Resend => {}
                }
            }

            let modal = Config::read(guild).await?.to_modal(())?;

            cpn.create_response(ctx, CreateInteractionResponse::Modal(modal))
                .await
                .map_err(Error::from)
        }
        CM_ABOUT => {
            let user = ctx.http.get_current_user().await?;
            let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
            let embed = CreateEmbed::new()
                .color(BOT_COLOR)
                .author(author)
                .description(include_str!(r"..\include\apply\about.txt"))
                .title("About Guild Applications");
            let message = CreateInteractionResponseMessage::new()
                .embed(embed)
                .ephemeral(true);

            cpn.create_response(ctx, CreateInteractionResponse::Message(message))
                .await
                .map_err(Error::from)
        }
        CM_ACCEPT | CM_DENY | CM_RESEND => {
            let Some(user) = custom_id.args.first() else {
                return Err(Error::MissingId(Value::User));
            };
            let Ok(user) = user.parse() else {
                return Err(Error::InvalidId(Value::User, user.to_string()));
            };
            let user = UserId::new(user);

            let status = match custom_id.name.as_str() {
                CM_ACCEPT => Status::Accepted,
                CM_DENY => Status::Denied,
                CM_RESEND => Status::Resend,
                n => return Err(Error::InvalidId(Value::Component, n.to_string())),
            };

            let modal = Form::read((guild, user)).await?.to_modal(status)?;

            cpn.create_response(ctx, CreateInteractionResponse::Modal(modal))
                .await
                .map_err(Error::from)
        }
        _ => Err(Error::InvalidId(Value::Component, custom_id.name)),
    }
}
pub async fn run_modal(ctx: &Context, mdl: &ModalInteraction) -> Result<()> {
    let custom_id = CustomId::try_from(mdl.data.custom_id.as_str())?;
    let Some(guild) = mdl.guild_id else {
        return Err(Error::MissingId(Value::Guild));
    };

    let o = &mdl.data.components;

    match custom_id.name.as_str() {
        MD_SUBMIT => {
            let mut form = Form {
                user: mdl.user.id,
                status: Status::Pending,
                reason: None,
                answers: (0..5)
                    .into_iter()
                    .filter_map(|n| get_input_text(o, &n.to_string()).ok().map(|s| (n, s)))
                    .collect(),
                anchor: None,
            };

            let mut message = CreateMessage::new().embed(form.to_embed(ctx, guild).await?);

            for button in form.to_button_array(false)? {
                message = message.button(button);
            }

            let config = Config::read(guild).await?;
            let message = config.channel.send_message(ctx, message).await?;

            form.anchor = Some(Anchor::try_from(message)?);
            form.write().await?;

            mdl.create_response(ctx, CreateInteractionResponse::Acknowledge)
                .await
                .map_err(Error::from)
        }
        MD_UPDATE => {
            let mut args = custom_id.args.iter();

            let Some(user) = args.next()else {
                return Err(Error::MissingId(Value::User));
            };
            let Ok(user) = user.parse() else {
                return Err(Error::InvalidId(Value::User, user.to_string()));
            };
            let user = UserId::new(user);

            let Some(status) = args.next() else {
                return Err(Error::MissingValue(Value::Other("Status")));
            };
            let Ok(status) = status.parse::<i32>() else {
                return Err(Error::InvalidValue(Value::Other("Status"), status.to_string()));
            };
            let status = Status::try_from(status)?;

            let reason = get_input_text(o, OP_REASON).ok();
            let mut form = Form::read((guild, user)).await?;

            form.update(ctx, status, reason).await?;
            mdl.create_response(ctx, CreateInteractionResponse::Acknowledge)
                .await
                .map_err(Error::from)
        }
        _ => Err(Error::InvalidId(Value::Modal, custom_id.name)),
    }
}
