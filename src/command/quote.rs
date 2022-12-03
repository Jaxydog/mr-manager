use crate::prelude::*;

pub const NAME: &str = "quote";

pub const OP_USER: &str = "user";
pub const OP_TEXT: &str = "text";

pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .default_member_permissions(Permissions::SEND_MESSAGES)
        .description("Quote something that a user said!")
        .dm_permission(false)
        .add_option(
            CreateCommandOption::new(CommandOptionType::User, OP_USER, "Who said it?")
                .required(true),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, OP_TEXT, "What did they say?")
                .max_length(256)
                .clone()
                .required(true),
        )
}

pub async fn run_command(ctx: &Context, cmd: &CommandInteraction) -> Result<()> {
    let o = &cmd.data.options();

    let user = get_user(o, OP_USER)?;
    let user = ctx.http.get_user(user.0.id).await?;
    let text = get_str(o, OP_TEXT)?;

    let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
    let embed = CreateEmbed::new()
        .author(author)
        .color(user.accent_colour.unwrap_or(BOT_COLOR))
        .description(format!("> {text}"));

    let message = CreateInteractionResponseMessage::new().embed(embed);
    cmd.create_response(ctx, CreateInteractionResponse::Message(message))
        .await
        .map_err(Error::from)
}
