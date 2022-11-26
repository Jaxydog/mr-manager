use crate::prelude::*;

pub const NAME: &str = "ping";

pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .default_member_permissions(Permissions::USE_APPLICATION_COMMANDS)
        .description("Check the bot's API response time")
        .dm_permission(true)
}

pub async fn run_command(ctx: &Context, cmd: &CommandInteraction) -> Result<()> {
    let mut embed = CreateEmbed::new().color(BOT_COLOR).title("Calculating...");
    let msg = CreateInteractionResponseMessage::new()
        .embed(embed.clone())
        .ephemeral(true);

    cmd.create_response(ctx, CreateInteractionResponse::Message(msg))
        .await?;

    let response = cmd.get_response(ctx).await?;
    let sent = response.id.created_at().timestamp_millis();
    let received = cmd.id.created_at().timestamp_millis();
    let delay = sent - received;

    embed = embed.title(format!("Pong! ({delay}ms)"));
    cmd.edit_response(ctx, EditInteractionResponse::new().embed(embed))
        .await
        .map(|_| ())
        .map_err(Error::from)
}
