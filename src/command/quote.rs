use crate::prelude::*;

pub const NAME: &str = "quote";

pub const OPTION_USER: &str = "user";
pub const OPTION_TEXT: &str = "text";

pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .default_member_permissions(Permissions::SEND_MESSAGES)
        .description("Quote something that a user said!")
        .dm_permission(false)
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, OPTION_USER, "Who said it?")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, OPTION_TEXT, "What did they say?")
                .max_length(256)
                .clone()
                .required(true),
        )
}

pub async fn run_command(http: &Http, cmd: &CommandInteraction) -> Result<()> {
    let o = &cmd.data.options();

    let user = get_user(o, OPTION_USER)?;
    let user = http.get_user(user.0.id).await?;

    if user == cmd.user {
        return Err(Error::Other("You cannot quote yourself"));
    }
    if user.bot {
        return Err(Error::Other("You cannot quote a bot"));
    }

    let text = get_str(o, OPTION_TEXT)?;

    let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
    let embed = CreateEmbed::new()
        .author(author)
        .color(user.accent_colour.unwrap_or(BOT_COLOR))
        .description(format!("> {text}"));

    let message = CreateInteractionResponseMessage::new().embed(embed);
    cmd.create_response(http, CreateInteractionResponse::Message(message))
        .await
        .map_err(Error::from)
}
