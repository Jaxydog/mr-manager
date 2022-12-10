use crate::prelude::*;

pub const NAME: &str = "ping";

pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .default_member_permissions(Permissions::USE_APPLICATION_COMMANDS)
        .description("Check the bot's API response time")
        .dm_permission(true)
}

pub async fn run_command(http: &Http, cmd: &CommandInteraction) -> Result<()> {
    let mut embed = CreateEmbed::new().color(BOT_COLOR).title("Calculating...");
    let message = CreateInteractionResponseMessage::new()
        .embed(embed.clone())
        .ephemeral(true);

    cmd.create_response(http, CreateInteractionResponse::Message(message))
        .await?;

    let res = cmd.get_response(http).await?;
    let sent = res.id.created_at().timestamp_millis();
    let received = cmd.id.created_at().timestamp_millis();
    let ms = sent - received;

    embed = embed.title(format!("Pong! ({ms}ms)"));
    cmd.edit_response(http, EditInteractionResponse::new().embed(embed))
        .await?;

    Ok(())
}
