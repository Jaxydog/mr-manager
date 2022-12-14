use crate::prelude::*;

use self::form::*;
use self::input::*;
use self::output::*;

pub mod form;
pub mod input;
pub mod output;

pub const NAME: &str = "poll";

pub const GROUP_INPUT: &str = "input";

pub const SUB_CREATE: &str = "create";
pub const SUB_DISCARD: &str = "discard";
pub const SUB_MODIFY: &str = "modify";
pub const SUB_PREVIEW: &str = "preview";
pub const SUB_SEND: &str = "send";
pub const SUB_CLOSE: &str = "close";

pub const OPTION_KIND: &str = "kind";
pub const OPTION_TITLE: &str = "title";
pub const OPTION_DESCRIPTION: &str = "description";
pub const OPTION_HOURS: &str = "hours";
pub const OPTION_IMAGE: &str = "image_link";
pub const OPTION_HIDE_MEMBERS: &str = "hidden_members";
pub const OPTION_HIDE_RESULTS: &str = "hidden_results";
pub const OPTION_FORCE: &str = "force";
pub const OPTION_LABEL: &str = "label";
pub const OPTION_EMOJI: &str = "emoji";
pub const OPTION_PLACEHOLDER: &str = "placeholder";

#[allow(clippy::too_many_lines)]
pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .default_member_permissions(Permissions::SEND_MESSAGES)
        .description("Create or manage polls")
        .dm_permission(false)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                SUB_CREATE,
                "Creates a new poll",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    OPTION_KIND,
                    "The type of poll",
                )
                .add_int_choice(Kind::Choice.to_string(), Kind::Choice as i32)
                .add_int_choice(Kind::Response.to_string(), Kind::Response as i32)
                .add_int_choice(Kind::Raffle.to_string(), Kind::Raffle as i32)
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_TITLE,
                    "The title of the poll",
                )
                .max_length(256)
                .clone()
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_DESCRIPTION,
                    "The description of the poll",
                )
                .max_length(512)
                .clone()
                .required(true),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    OPTION_HOURS,
                    "The duration of the poll in hours",
                )
                .min_int_value(1)
                .max_int_value(240)
                .required(true),
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::String,
                OPTION_IMAGE,
                "The poll's image link; does not support GIFs",
            ))
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::Boolean,
                OPTION_HIDE_MEMBERS,
                "Whether the poll's member replies are anonymous",
            ))
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::Boolean,
                OPTION_HIDE_RESULTS,
                "Whether the poll's results are only visible to you",
            )),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                SUB_DISCARD,
                "Discards your poll",
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::Boolean,
                OPTION_FORCE,
                "Whether the poll should be discarded even if it has been sent",
            )),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommand,
                SUB_MODIFY,
                "Modifies your poll's content",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    OPTION_KIND,
                    "The type of poll; this will remove all existing inputs",
                )
                .add_int_choice(Kind::Choice.to_string(), Kind::Choice as i32)
                .add_int_choice(Kind::Response.to_string(), Kind::Response as i32)
                .add_int_choice(Kind::Raffle.to_string(), Kind::Raffle as i32),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_TITLE,
                    "The title of the poll",
                )
                .max_length(256)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_DESCRIPTION,
                    "The description of the poll",
                )
                .max_length(512)
                .clone(),
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::Integer,
                    OPTION_HOURS,
                    "The duration of the poll in hours",
                )
                .min_int_value(1)
                .max_int_value(240),
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::String,
                OPTION_IMAGE,
                "The poll's image link; does not support GIFs",
            ))
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::Boolean,
                OPTION_HIDE_MEMBERS,
                "Whether the poll's member replies are anonymous",
            ))
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::Boolean,
                OPTION_HIDE_RESULTS,
                "Whether the poll's results are only visible to you",
            )),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            SUB_PREVIEW,
            "Previews your poll",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            SUB_SEND,
            "Sends your poll",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::SubCommand,
            SUB_CLOSE,
            "Closes your poll",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::SubCommandGroup,
                GROUP_INPUT,
                "Create or manage poll inputs",
            )
            .add_sub_option(
                CreateCommandOption::new(
                    CommandOptionType::SubCommand,
                    SUB_CREATE,
                    "Creates a new poll input; does not work with Raffle polls",
                )
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        OPTION_LABEL,
                        "The label of the poll input",
                    )
                    .max_length(45)
                    .clone()
                    .required(true),
                )
                .add_sub_option(CreateCommandOption::new(
                    CommandOptionType::String,
                    OPTION_EMOJI,
                    "The emoji of the poll button; only works for Choice polls, and invalid values will be ignored",
                ))
                .add_sub_option(
                    CreateCommandOption::new(
                        CommandOptionType::String,
                        OPTION_PLACEHOLDER,
                        "The placeholder text of the poll field; only works for Response polls",
                    )
                    .max_length(45)
                    .clone(),
                ),
            )
            .add_sub_option(CreateCommandOption::new(
                CommandOptionType::SubCommand,
                SUB_DISCARD,
                "Discards poll inputs; does not work with Raffle polls",
            )),
        )
}

#[allow(clippy::too_many_lines)]
pub async fn run_command(http: &Http, cmd: &CommandInteraction) -> Result<()> {
    let guild = cmd.guild_id.ok_or(Error::MissingId(Value::Guild))?;
    let o = &cmd.data.options();

    if let Ok(o) = get_subcommand(o, SUB_CREATE) {
        if Form::read((guild, cmd.user.id)).is_ok() {
            return Err(Error::Other("You already have a poll"));
        }

        let kind = Kind::try_from(get_i64(o, OPTION_KIND)?)?;
        let title = get_str(o, OPTION_TITLE)?.to_string();
        let description = get_str(o, OPTION_DESCRIPTION)?.replace(['\r', '\n', '\t'], " ");
        let hours = get_i64(o, OPTION_HOURS)?;
        let image = get_str(o, OPTION_IMAGE).ok().map(str::to_string);
        let hide_members = get_bool(o, OPTION_HIDE_MEMBERS).unwrap_or_default();
        let hide_results = get_bool(o, OPTION_HIDE_RESULTS).unwrap_or_default();

        let content = Content {
            title,
            description,
            hours,
            image,
            hide_members,
            hide_results,
        };

        Form::new(cmd.user.id, kind, content).write(guild)?;

        let builder = CreateEmbed::new()
            .color(BOT_COLOR)
            .title("Created new poll!");
        let builder = CreateInteractionResponseMessage::new()
            .embed(builder)
            .ephemeral(true);

        cmd.create_response(http, CreateInteractionResponse::Message(builder))
            .await
            .map_err(Error::from)
    } else if let Ok(o) = get_subcommand(o, SUB_DISCARD) {
        let force = get_bool(o, OPTION_FORCE).unwrap_or_default();

        let Ok(form) = Form::read((guild, cmd.user.id)) else {
            return Err(Error::Other("You do not have a poll"));
        };
        if form.is_anchored() && !force {
            return Err(Error::Other("Your poll has already been sent"));
        }

        if let Ok(anchor) = form.anchor() {
            anchor.to_message(http).await?.delete(http).await?;
        }

        form.remove(guild)?;

        let builder = CreateEmbed::new()
            .color(BOT_COLOR)
            .title("Created new poll!");
        let builder = CreateInteractionResponseMessage::new()
            .embed(builder)
            .ephemeral(true);

        cmd.create_response(http, CreateInteractionResponse::Message(builder))
            .await
            .map_err(Error::from)
    } else if let Ok(o) = get_subcommand(o, SUB_MODIFY) {
        let Ok(mut form) = Form::read((guild, cmd.user.id)) else {
            return Err(Error::Other("You do not have a poll"));
        };
        if form.is_anchored() {
            return Err(Error::Other("Your poll has already been sent"));
        }

        if let Ok(kind) = get_i64(o, OPTION_KIND) {
            let kind = Kind::try_from(kind)?;
            form.kind = kind;
            form.inputs.clear();
        }
        if let Ok(title) = get_str(o, OPTION_TITLE).map(str::to_string) {
            form.content.title = title;
        }
        if let Ok(description) = get_str(o, OPTION_DESCRIPTION).map(str::to_string) {
            form.content.description = description;
        }
        if let Ok(hours) = get_i64(o, OPTION_HOURS) {
            form.content.hours = hours;
        }
        if let Ok(image) = get_str(o, OPTION_IMAGE).map(str::to_string) {
            form.content.image = Some(image);
        }
        if let Ok(hide_members) = get_bool(o, OPTION_HIDE_MEMBERS) {
            form.content.hide_members = hide_members;
        }
        if let Ok(hide_results) = get_bool(o, OPTION_HIDE_RESULTS) {
            form.content.hide_results = hide_results;
        }

        form.write(guild)?;

        let builder = CreateEmbed::new()
            .color(BOT_COLOR)
            .title("Modified poll content!");
        let builder = CreateInteractionResponseMessage::new()
            .embed(builder)
            .ephemeral(true);

        cmd.create_response(http, CreateInteractionResponse::Message(builder))
            .await
            .map_err(Error::from)
    } else if get_subcommand(o, SUB_PREVIEW).is_ok() {
        let Ok(form) = Form::read((guild, cmd.user.id)) else {
            return Err(Error::Other("You do not have a poll"));
        };
        if form.is_anchored() {
            return Err(Error::Other("Your poll has already been sent"));
        }

        let builder = form.as_embed(http, ()).await?;
        let mut builder = CreateInteractionResponseMessage::new()
            .embed(builder)
            .ephemeral(true);

        for button in form.as_buttons(true, ()) {
            builder = builder.button(button);
        }

        cmd.create_response(http, CreateInteractionResponse::Message(builder))
            .await
            .map_err(Error::from)
    } else if get_subcommand(o, SUB_SEND).is_ok() {
        let Ok(mut form) = Form::read((guild, cmd.user.id)) else {
            return Err(Error::Other("You do not have a poll"));
        };
        if form.is_anchored() {
            return Err(Error::Other("Your poll has already been sent"));
        }
        if form.kind != Kind::Raffle && form.inputs.is_empty() {
            return Err(Error::Other("Your poll does not have any inputs"));
        }
        if form.kind == Kind::Choice && form.inputs.len() <= 1 {
            return Err(Error::Other("Your poll must have more than one input"));
        }

        // disable force sending for now
        form.send(http, guild, cmd.channel_id, true).await?;

        let builder = CreateEmbed::new()
            .color(BOT_COLOR)
            .title("Your poll has been published!");
        let builder = CreateInteractionResponseMessage::new()
            .embed(builder)
            .ephemeral(true);

        cmd.create_response(http, CreateInteractionResponse::Message(builder))
            .await
            .map_err(Error::from)
    } else if get_subcommand(o, SUB_CLOSE).is_ok() {
        let Ok(form) = Form::read((guild, cmd.user.id)) else {
            return Err(Error::Other("You do not have a poll"));
        };
        if form.is_floating() {
            return Err(Error::Other("Your poll has not been sent"));
        }

        form.close(http).await?;

        let builder = CreateEmbed::new()
            .color(BOT_COLOR)
            .title("Your poll has been closed!");
        let builder = CreateInteractionResponseMessage::new()
            .embed(builder)
            .ephemeral(true);

        cmd.create_response(http, CreateInteractionResponse::Message(builder))
            .await
            .map_err(Error::from)
    } else if let Ok(o) = get_subcommand_group(o, GROUP_INPUT) {
        if let Ok(o) = get_subcommand(o, SUB_CREATE) {
            let label = get_str(o, OPTION_LABEL)?;

            let Ok(mut form) = Form::read((guild, cmd.user.id)) else {
                return Err(Error::Other("You do not have a poll"));
            };
            if form.is_anchored() {
                return Err(Error::Other("Your poll has already been sent"));
            }
            if form.inputs.len() >= Input::max_count(form.kind) {
                return Err(Error::Other("No more inputs may be added"));
            }
            if form.inputs.iter().any(|input| input.label() == label) {
                return Err(Error::Other("The given input already exists"));
            }

            let input = match form.kind {
                Kind::Choice => {
                    let label = label.to_string();
                    let data = get_str(o, OPTION_EMOJI)
                        .ok()
                        .and_then(|s| ReactionType::try_from(s).ok());

                    Input::Choice(InputData { label, data })
                }
                Kind::Response => {
                    let label = label.to_string();
                    let data = get_str(o, OPTION_PLACEHOLDER).ok().map(str::to_string);

                    Input::Response(InputData { label, data })
                }
                Kind::Raffle => return Err(Error::Other("Raffle polls do not support inputs")),
            };

            form.inputs.push(input);
            form.write(guild)?;

            let builder = CreateEmbed::new()
                .color(BOT_COLOR)
                .title(format!("Added input '{label}'!"));
            let builder = CreateInteractionResponseMessage::new()
                .embed(builder)
                .ephemeral(true);

            cmd.create_response(http, CreateInteractionResponse::Message(builder))
                .await
                .map_err(Error::from)
        } else if get_subcommand(o, SUB_DISCARD).is_ok() {
            let Ok(form) = Form::read((guild, cmd.user.id)) else {
                return Err(Error::Other("You do not have a poll"));
            };
            if form.is_anchored() {
                return Err(Error::Other("Your poll has already been sent"));
            }
            if form.inputs.is_empty() {
                return Err(Error::Other("Your poll does not have any inputs"));
            }

            let builder = form.as_remove_message(false);

            cmd.create_response(http, CreateInteractionResponse::Message(builder))
                .await
                .map_err(Error::from)
        } else {
            Err(Error::InvalidId(Value::Command, cmd.data.name.clone()))
        }
    } else {
        Err(Error::InvalidId(Value::Command, cmd.data.name.clone()))
    }
}
#[allow(clippy::map_entry, clippy::too_many_lines)]
pub async fn run_component(http: &Http, cpn: &mut ComponentInteraction) -> Result<()> {
    let custom_id = CustomId::try_from(cpn.data.custom_id.as_str())?;
    let guild = cpn.guild_id.ok_or(Error::MissingId(Value::Guild))?;

    match custom_id.name.as_str() {
        BUTTON_REMOVE => {
            let Some(user) = custom_id.args.first() else {
                return Err(Error::MissingId(Value::User));
            };
            let Ok(user) = user.parse() else {
                return Err(Error::InvalidId(Value::User, user.to_string()));
            };
            let user = UserId::new(user);

            let Some(index) = custom_id.args.get(1) else {
                return Err(Error::MissingId(Value::Data));
            };
            let Ok(index) = index.parse() else {
                return Err(Error::InvalidId(Value::Data, index.clone()));
            };

            let mut form = Form::read((guild, user))?;

            if user != cpn.user.id {
                return Err(Error::Other("You cannot modify another user's poll"));
            }
            if form.inputs.len() <= index {
                return Err(Error::InvalidId(Value::Data, index.to_string()));
            }

            form.inputs.remove(index);
            form.write(guild)?;

            let builder = if form.inputs.is_empty() {
                let builder = CreateEmbed::new()
                    .color(BOT_COLOR)
                    .title("All inputs removed!");

                CreateInteractionResponseMessage::new()
                    .embed(builder)
                    .ephemeral(true)
            } else {
                form.as_remove_message(false)
            };

            cpn.create_response(http, CreateInteractionResponse::Message(builder))
                .await
                .map_err(Error::from)
        }
        BUTTON_CHOICE => {
            let Some(user) = custom_id.args.first() else {
                return Err(Error::MissingId(Value::User));
            };
            let Ok(user) = user.parse() else {
                return Err(Error::InvalidId(Value::User, user.to_string()));
            };
            let user = UserId::new(user);

            let Some(index) = custom_id.args.get(1) else {
                return Err(Error::MissingId(Value::Data));
            };
            let Ok(index) = index.parse() else {
                return Err(Error::InvalidId(Value::Data, index.clone()));
            };

            let mut form = Form::read((guild, user))?;

            if user == cpn.user.id {
                return Err(Error::Other("You cannot respond to your own poll"));
            }
            if form.inputs.len() <= index {
                return Err(Error::InvalidId(Value::Data, index.to_string()));
            }

            let builder = CreateEmbed::new().color(BOT_COLOR);

            if let Some(Reply::Choice(data)) = form.replies.get(&cpn.user.id) {
                if index == *data {
                    form.replies.remove(&cpn.user.id);
                    form.write(guild)?;

                    let builder = CreateInteractionResponseMessage::new()
                        .embed(builder.title("Your response has been removed!"))
                        .ephemeral(true);

                    return cpn
                        .create_response(http, CreateInteractionResponse::Message(builder))
                        .await
                        .map_err(Error::from);
                }
            }

            form.replies.insert(cpn.user.id, Reply::Choice(index));
            form.write(guild)?;

            let builder = CreateInteractionResponseMessage::new()
                .embed(builder.title("Your response has been recorded!"))
                .ephemeral(true);

            cpn.create_response(http, CreateInteractionResponse::Message(builder))
                .await
                .map_err(Error::from)
        }
        BUTTON_RESPONSE => {
            let Some(user) = custom_id.args.first() else {
                return Err(Error::MissingId(Value::User));
            };
            let Ok(user) = user.parse() else {
                return Err(Error::InvalidId(Value::User, user.to_string()));
            };
            let user = UserId::new(user);

            let form = Form::read((guild, user))?;

            if user == cpn.user.id {
                return Err(Error::Other("You cannot respond to your own form"));
            }

            let builder = form.try_as_modal(())?;

            cpn.create_response(http, CreateInteractionResponse::Modal(builder))
                .await
                .map_err(Error::from)
        }
        BUTTON_RAFFLE => {
            let Some(user) = custom_id.args.first() else {
                return Err(Error::MissingId(Value::User));
            };
            let Ok(user) = user.parse() else {
                return Err(Error::InvalidId(Value::User, user.to_string()));
            };
            let user = UserId::new(user);

            let mut form = Form::read((guild, user))?;

            if user == cpn.user.id {
                return Err(Error::Other("You cannot respond to your own form"));
            }

            let title = if form.replies.contains_key(&cpn.user.id) {
                form.replies.remove(&cpn.user.id);

                "You have been removed from the raffle"
            } else {
                form.replies.insert(cpn.user.id, Reply::Raffle);

                "You have been added to the raffle"
            };

            form.write(guild)?;

            let builder = CreateEmbed::new().color(BOT_COLOR).title(title);
            let builder = CreateInteractionResponseMessage::new()
                .embed(builder)
                .ephemeral(true);

            cpn.create_response(http, CreateInteractionResponse::Message(builder))
                .await
                .map_err(Error::from)
        }
        BUTTON_RESULTS => {
            let Some(user) = custom_id.args.first() else {
                return Err(Error::MissingId(Value::User));
            };
            let Ok(user) = user.parse() else {
                return Err(Error::InvalidId(Value::User, user.to_string()));
            };
            let user = UserId::new(user);

            let Some(message) = custom_id.args.get(1) else {
                return Err(Error::MissingId(Value::Message));
            };
            let Ok(message) = message.parse() else {
                return Err(Error::InvalidId(Value::Message, message.to_string()));
            };
            let message = MessageId::new(message);

            let mut form = Form::read((guild, user, message))?;

            if form.content.hide_results && cpn.user.id != user {
                return Err(Error::Other("The results of this poll are private"));
            };

            let output = form.output().clone();
            let builder = output.as_embed(http, (form, 1)).await?;
            let mut builder = CreateInteractionResponseMessage::new()
                .embed(builder)
                .ephemeral(true);

            for button in output.as_buttons(false, (user, message, 1)) {
                builder = builder.button(button);
            }

            cpn.create_response(http, CreateInteractionResponse::Message(builder))
                .await
                .map_err(Error::from)
        }
        BUTTON_LAST | BUTTON_NEXT => {
            let Some(user) = custom_id.args.first() else {
                return Err(Error::MissingId(Value::User));
            };
            let Ok(user) = user.parse() else {
                return Err(Error::InvalidId(Value::User, user.to_string()));
            };
            let user = UserId::new(user);

            let Some(message) = custom_id.args.get(1) else {
                return Err(Error::MissingId(Value::Message));
            };
            let Ok(message) = message.parse() else {
                return Err(Error::InvalidId(Value::Message, message.to_string()));
            };
            let message = MessageId::new(message);

            let Some(page) = custom_id.args.get(2) else {
                return Err(Error::MissingId(Value::Data));
            };
            let Ok(page) = page.parse::<usize>() else {
                return Err(Error::InvalidId(Value::Data, page.clone()));
            };
            let page = match custom_id.name.as_str() {
                BUTTON_LAST => page.saturating_sub(1),
                BUTTON_NEXT => page.saturating_add(1),
                _ => return Err(Error::InvalidId(Value::Component, custom_id.name)),
            };

            let mut form = Form::read((guild, user, message))?;

            if form.content.hide_results && cpn.user.id != user {
                return Err(Error::Other("The results of this poll are private"));
            };

            let output = form.output().clone();
            let builder = output.as_embed(http, (form, page)).await?;
            let mut builder = CreateInteractionResponseMessage::new()
                .embed(builder)
                .ephemeral(true);

            for button in output.as_buttons(false, (user, message, page)) {
                builder = builder.button(button);
            }

            cpn.create_response(http, CreateInteractionResponse::Message(builder))
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

    if custom_id.name != MODAL_SUBMIT {
        return Err(Error::InvalidId(Value::Modal, custom_id.name));
    }

    let Some(user) = custom_id.args.first() else {
        return Err(Error::MissingId(Value::User));
    };
    let Ok(user) = user.parse() else {
        return Err(Error::InvalidId(Value::User, user.to_string()));
    };
    let user = UserId::new(user);

    let mut form = Form::read((guild, user))?;

    if user == mdl.user.id {
        return Err(Error::Other("You cannot respond to your own form"));
    }

    let mut answers = Vec::with_capacity(form.inputs.len());

    for index in 0..form.inputs.len() {
        let text = get_input_text(o, &index.to_string());

        answers.push(text.unwrap_or_else(|_| "N/A".to_string()));
    }

    form.replies.insert(mdl.user.id, Reply::Response(answers));
    form.write(guild)?;

    let builder = CreateEmbed::new()
        .color(BOT_COLOR)
        .title("Your response has been recorded");
    let builder = CreateInteractionResponseMessage::new()
        .embed(builder)
        .ephemeral(true);

    mdl.create_response(http, CreateInteractionResponse::Message(builder))
        .await
        .map_err(Error::from)
}

pub async fn check(http: &Http) -> Result<()> {
    let Ok(mut active) = Active::read(()) else {
        return Ok(());
    };
    let mut invalid = vec![];

    for key in &active.0 {
        let Ok(form) = Form::read(*key) else {
            invalid.push(*key);
            continue;
        };
        let Ok(anchor) = form.anchor() else {
            invalid.push(*key);
            continue;
        };

        let sent = anchor.message.created_at().timestamp_millis();
        let duration = form.content.hours * 60 * 60 * 1000;

        if Utc::now().timestamp_millis() >= sent + duration {
            form.close(http).await?;
        }
    }

    if !invalid.is_empty() {
        for key in invalid {
            active.0.remove(&key);
        }

        active.write(())?;
    }

    Ok(())
}
