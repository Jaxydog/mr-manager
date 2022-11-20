use serenity::{
    builder::{
        CreateCommand, CreateEmbed, CreateEmbedAuthor, CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
    model::{prelude::CommandInteraction, Permissions},
    prelude::Context,
};

use crate::{utility::Result, DEFAULT_COLOR};

pub const NAME: &str = "data";

pub fn register() -> CreateCommand {
    CreateCommand::new(NAME)
        .description("Displays information about data privacy and usage")
        .default_member_permissions(Permissions::USE_APPLICATION_COMMANDS)
        .dm_permission(true)
}

pub async fn run(ctx: &Context, cmd: &CommandInteraction) -> Result<()> {
    let user = ctx.http.get_current_user().await?;
    let embed = CreateEmbed::new()
        .author(CreateEmbedAuthor::new(user.tag()).icon_url(user.face()))
        .color(DEFAULT_COLOR)
        .description(include_str!(r"..\include\data.txt"))
        .title("About Stored Data");

    cmd.create_response(
        &ctx.http,
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .embed(embed)
                .ephemeral(true),
        ),
    )
    .await?;

    Ok(())
}
