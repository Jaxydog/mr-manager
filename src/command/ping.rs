use serenity::{
    builder::{
        CreateCommand, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage,
        EditInteractionResponse,
    },
    model::{prelude::CommandInteraction, Permissions},
    prelude::Context,
};

use crate::{utility::Result, DEFAULT_COLOR};

pub const NAME: &str = "ping";

pub fn register() -> CreateCommand {
    CreateCommand::new(NAME)
        .description("Tests the bot's API connection")
        .default_member_permissions(Permissions::USE_APPLICATION_COMMANDS)
        .dm_permission(true)
}

pub async fn run(ctx: &Context, cmd: &CommandInteraction) -> Result<()> {
    let mut embed = CreateEmbed::new()
        .color(DEFAULT_COLOR)
        .title("Calculating...");

    cmd.create_response(
        &ctx.http,
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .embed(embed.clone())
                .ephemeral(true),
        ),
    )
    .await?;

    let response = cmd.get_response(&ctx.http).await?;
    let sent = response.id.created_at().timestamp_millis();
    let received = cmd.id.created_at().timestamp_millis();
    let delay = sent - received;

    embed = embed.title(format!("Pong! ({delay}ms)"));

    cmd.edit_response(&ctx.http, EditInteractionResponse::new().embed(embed))
        .await?;

    Ok(())
}
