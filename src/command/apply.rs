use crate::prelude::*;

pub use self::config::*;
pub use self::form::*;

pub mod config;
pub mod form;

pub const NAME: &str = "apply";

pub const SUB_CONFIG: &str = "config";
pub const SUB_MODIFY: &str = "modify";
pub const SUB_UPDATE: &str = "update";
pub const SUB_REMOVE: &str = "remove";

pub const OPTION_TITLE: &str = "title";
pub const OPTION_DESCRIPTION: &str = "description";
pub const OPTION_THUMB_LINK: &str = "thumbnail_link";
pub const OPTION_CHANNEL: &str = "output_channel";
pub const OPTION_ROLE: &str = "acceptance_role";
pub const OPTION_QUESTION_1: &str = "question_1";
pub const OPTION_QUESTION_2: &str = "question_2";
pub const OPTION_QUESTION_3: &str = "question_3";
pub const OPTION_QUESTION_4: &str = "question_4";
pub const OPTION_QUESTION_5: &str = "question_5";
pub const OPTION_USER: &str = "user";
pub const OPTION_STATUS: &str = "status";
pub const OPTION_OVERWRITE: &str = "overwrite";

#[allow(clippy::too_many_lines)]
pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .default_member_permissions(Permissions::MODERATE_MEMBERS)
        .description("Manage guild applications")
        .dm_permission(false)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                SUB_CONFIG,
                "Configure guild applications",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_TITLE,
                    "The title of the guild application embed",
                )
                .max_length(256)
                .clone()
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_DESCRIPTION,
                    "The description of the guild application embed",
                )
                .max_length(4096)
                .clone()
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_THUMB_LINK,
                    "The thumbnail link of the guild application embed",
                )
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    OPTION_CHANNEL,
                    "The output channel for submitted forms",
                )
                .channel_types(vec![ChannelType::Text])
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Role,
                    OPTION_ROLE,
                    "The role given to members that are accepted",
                )
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_QUESTION_1,
                    "The first question on the application",
                )
                .max_length(45)
                .clone()
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_QUESTION_2,
                    "The second question on the application",
                )
                .max_length(45)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_QUESTION_3,
                    "The third question on the application",
                )
                .max_length(45)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_QUESTION_4,
                    "The fourth question on the application",
                )
                .max_length(45)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_QUESTION_5,
                    "The fifth question on the application",
                )
                .max_length(45)
                .clone(),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                SUB_MODIFY,
                "Modify the guild application configuration",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_TITLE,
                    "The title of the guild application embed",
                )
                .max_length(256)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_DESCRIPTION,
                    "The description of the guild application embed",
                )
                .max_length(4096)
                .clone(),
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::String,
                OPTION_THUMB_LINK,
                "The thumbnail link of the guild application embed",
            ))
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Channel,
                    OPTION_CHANNEL,
                    "The output channel for submitted forms",
                )
                .channel_types(vec![ChannelType::Text]),
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::Role,
                OPTION_ROLE,
                "The role given to members that are accepted",
            ))
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_QUESTION_1,
                    "The first question on the application",
                )
                .max_length(45)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_QUESTION_2,
                    "The second question on the application",
                )
                .max_length(45)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_QUESTION_3,
                    "The third question on the application",
                )
                .max_length(45)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_QUESTION_4,
                    "The fourth question on the application",
                )
                .max_length(45)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_QUESTION_5,
                    "The fifth question on the application",
                )
                .max_length(45)
                .clone(),
            ),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                SUB_UPDATE,
                "Update a member's submitted application",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::User,
                    OPTION_USER,
                    "The guild member that submitted the application",
                )
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    OPTION_STATUS,
                    "The new status of the application",
                )
                .add_int_choice(Status::Accepted.to_string(), Status::Accepted as i32)
                .add_int_choice(Status::Denied.to_string(), Status::Denied as i32)
                .add_int_choice(Status::Resend.to_string(), Status::Resend as i32)
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_REASON,
                    "The reason for the update",
                )
                .max_length(256)
                .clone(),
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::Boolean,
                OPTION_OVERWRITE,
                "Whether to overwrite a finalized application (default false)",
            )),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                SUB_REMOVE,
                "Remove a member's submitted application",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::User,
                    OPTION_USER,
                    "The guild member that submitted the application",
                )
                .required(true),
            ),
        )
}

#[allow(clippy::too_many_lines)]
pub async fn run_command(http: &Http, cmd: &CommandInteraction) -> Result<()> {
    let guild = cmd.guild_id.ok_or(Error::MissingId(Value::Guild))?;
    let o = &cmd.data.options();

    if let Ok(o) = get_subcommand(o, SUB_CONFIG) {
        let title = get_str(o, OPTION_TITLE)?;
        let description = get_str(o, OPTION_DESCRIPTION)?.replace(r"\n", "\n");
        let thumbnail = get_str(o, OPTION_THUMB_LINK)?;
        let questions = [
            get_str(o, OPTION_QUESTION_1),
            get_str(o, OPTION_QUESTION_2),
            get_str(o, OPTION_QUESTION_3),
            get_str(o, OPTION_QUESTION_4),
            get_str(o, OPTION_QUESTION_5),
        ]
        .into_iter()
        .filter_map(Result::ok)
        .collect();

        let mut config = Config::new(
            get_channel(o, OPTION_CHANNEL)?.id,
            get_role(o, OPTION_ROLE)?.id,
            Content::new(title, description, thumbnail, questions),
        );

        config.send(http, guild, cmd.channel_id).await?;

        let embed = CreateEmbed::new()
            .color(BOT_COLOR)
            .title("Configured applications!");
        let message = CreateInteractionResponseMessage::new()
            .embed(embed)
            .ephemeral(true);

        cmd.create_response(http, CreateInteractionResponse::Message(message))
            .await
            .map_err(Error::from)
    } else if let Ok(o) = get_subcommand(o, SUB_MODIFY) {
        let mut config = Config::read(guild)?;
        let mut update = false;

        if let Ok(title) = get_str(o, OPTION_TITLE) {
            config.content.title = title.to_string();
            update = true;
        }
        if let Ok(description) = get_str(o, OPTION_DESCRIPTION) {
            config.content.description = description.to_string();
            update = true;
        }
        if let Ok(thumbnail) = get_str(o, OPTION_THUMB_LINK) {
            config.content.thumbnail = thumbnail.to_string();
            update = true;
        }
        if let Ok(question) = get_str(o, OPTION_QUESTION_1) {
            config.content.questions[0] = question.to_string();
            update = true;
        }
        if let Ok(question) = get_str(o, OPTION_QUESTION_2) {
            config.content.questions[1] = question.to_string();
            update = true;
        }
        if let Ok(question) = get_str(o, OPTION_QUESTION_3) {
            config.content.questions[2] = question.to_string();
            update = true;
        }
        if let Ok(question) = get_str(o, OPTION_QUESTION_4) {
            config.content.questions[3] = question.to_string();
            update = true;
        }
        if let Ok(question) = get_str(o, OPTION_QUESTION_5) {
            config.content.questions[4] = question.to_string();
            update = true;
        }

        if let Ok(channel) = get_channel(o, OPTION_CHANNEL) {
            config.channel = channel.id;
        }
        if let Ok(role) = get_role(o, OPTION_ROLE) {
            config.role = role.id;
        }

        if update {
            config.send(http, guild, cmd.channel_id).await?;
        } else {
            config.try_write(())?;
        }

        let embed = CreateEmbed::new()
            .color(BOT_COLOR)
            .title("Updated application configuration!");
        let message = CreateInteractionResponseMessage::new()
            .embed(embed)
            .ephemeral(true);

        cmd.create_response(http, CreateInteractionResponse::Message(message))
            .await
            .map_err(Error::from)
    } else if let Ok(o) = get_subcommand(o, SUB_UPDATE) {
        let (user, _) = get_user(o, OPTION_USER)?;
        let status = Status::try_from(get_i64(o, OPTION_STATUS)?)?;
        let reason = get_str(o, OPTION_REASON).ok();
        let overwrite = get_bool(o, OPTION_OVERWRITE).unwrap_or(false);

        let config = Config::read(guild)?;
        let mut form = Form::read((guild, user.id))?;

        if form.status == status {
            return Err(Error::Other("The application already has this status"));
        }
        if !overwrite && form.status != Status::Pending {
            return Err(Error::Other("The user's application is already finalized"));
        }

        form.update(http, guild, config.role, status, reason)
            .await?;

        let embed = CreateEmbed::new()
            .color(BOT_COLOR)
            .title("Updated user application!");
        let message = CreateInteractionResponseMessage::new()
            .embed(embed)
            .ephemeral(true);

        cmd.create_response(http, CreateInteractionResponse::Message(message))
            .await
            .map_err(Error::from)
    } else if let Ok(o) = get_subcommand(o, SUB_REMOVE) {
        let (user, _) = get_user(o, OPTION_USER)?;
        let config = Config::read(guild)?;

        let form = Form::read((guild, user.id))?;
        let mut member = guild.member(http, user.id).await?;

        if let Ok(anchor) = form.anchor() {
            anchor.to_message(http).await?.delete(http).await?;
        }
        if member.roles.contains(&config.role) {
            member.remove_role(http, config.role).await?;
        }

        form.try_remove(())?;

        let embed = CreateEmbed::new()
            .color(BOT_COLOR)
            .title("Removed user application!");
        let message = CreateInteractionResponseMessage::new()
            .embed(embed)
            .ephemeral(true);

        cmd.create_response(http, CreateInteractionResponse::Message(message))
            .await
            .map_err(Error::from)
    } else {
        Err(Error::InvalidId(Value::Command, cmd.data.name.clone()))
    }
}
pub async fn run_component(http: &Http, cpn: &mut ComponentInteraction) -> Result<()> {
    let custom_id = CustomId::try_from(cpn.data.custom_id.as_str())?;
    let guild = cpn.guild_id.ok_or(Error::MissingId(Value::Guild))?;

    match custom_id.name.as_str() {
        BUTTON_MODAL => {
            if let Ok(form) = Form::read((guild, cpn.user.id)) {
                match form.status {
                    Status::Pending => return Err(Error::Other("Your application is pending")),
                    Status::Accepted => return Err(Error::Other("Your application was accepted")),
                    Status::Denied => return Err(Error::Other("Your application was denied")),
                    Status::Resend => {}
                }
            }

            let modal = Config::read(guild)?.as_modal(());

            cpn.create_response(http, CreateInteractionResponse::Modal(modal))
                .await
                .map_err(Error::from)
        }
        BUTTON_ABOUT => {
            let user = http.get_current_user().await?;
            let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
            let embed = CreateEmbed::new()
                .author(author)
                .color(BOT_COLOR)
                .description(include_str!(r"..\include\apply\about.txt"))
                .title("About Guild Applications");

            let message = CreateInteractionResponseMessage::new()
                .embed(embed)
                .ephemeral(true);

            cpn.create_response(http, CreateInteractionResponse::Message(message))
                .await
                .map_err(Error::from)
        }
        BUTTON_ACCEPT | BUTTON_DENY | BUTTON_RESEND => {
            let Some(user) = custom_id.args.first() else {
                return Err(Error::MissingId(Value::User));
            };
            let Ok(user) = user.parse() else {
                return Err(Error::InvalidId(Value::User, user.to_string()));
            };
            let user = UserId::new(user);

            let status = match custom_id.name.as_str() {
                BUTTON_ACCEPT => Status::Accepted,
                BUTTON_DENY => Status::Denied,
                BUTTON_RESEND => Status::Resend,
                _ => return Err(Error::InvalidId(Value::Component, custom_id.name)),
            };

            let modal = Form::read((guild, user))?.as_modal(status);

            cpn.create_response(http, CreateInteractionResponse::Modal(modal))
                .await
                .map_err(Error::from)
        }
        _ => Err(Error::InvalidId(Value::Component, custom_id.name)),
    }
}
pub async fn run_modal(http: &Http, mdl: &ModalInteraction) -> Result<()> {
    let custom_id = CustomId::try_from(mdl.data.custom_id.as_str())?;
    let guild = mdl.guild_id.ok_or(Error::MissingId(Value::Guild))?;
    let o = &mdl.data.components;

    let config = Config::read(guild)?;

    match custom_id.name.as_str() {
        MODAL_SUBMIT => {
            let answers = (0..5_u8)
                .filter_map(|n| get_input_text(o, &n.to_string()).ok())
                .collect();

            let mut form = Form::new(mdl.user.id, answers);

            form.send(http, guild, config.channel).await?;

            mdl.create_response(http, CreateInteractionResponse::Acknowledge)
                .await
                .map_err(Error::from)
        }
        MODAL_UPDATE => {
            let mut args = custom_id.args.iter();

            let Some(user) = args.next() else {
                return Err(Error::MissingId(Value::User))
            };
            let Ok(user) = user.parse() else {
                return Err(Error::InvalidId(Value::User, user.to_string()));
            };
            let user = UserId::new(user);

            let Some(status) = args.next() else {
                return Err(Error::MissingValue(Value::Other("Status")));
            };
            let Ok(status) = status.parse::<i64>() else {
                return Err(Error::InvalidValue(Value::Other("Status"), status.to_string()))
            };
            let status = Status::try_from(status)?;

            let reason = get_input_text(o, OPTION_REASON).ok();
            let mut form = Form::read((guild, user))?;

            form.update(http, guild, config.role, status, reason)
                .await?;

            mdl.create_response(http, CreateInteractionResponse::Acknowledge)
                .await
                .map_err(Error::from)
        }
        _ => Err(Error::InvalidId(Value::Modal, custom_id.name)),
    }
}
