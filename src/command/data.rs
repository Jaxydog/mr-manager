use crate::prelude::*;

pub const NAME: &str = "data";

pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .description("Displays information about data usage and privacy")
        .default_member_permissions(Permissions::USE_APPLICATION_COMMANDS)
        .dm_permission(true)
}

pub async fn run_command(ctx: &Context, cmd: &CommandInteraction) -> Result<()> {
    let user = ctx.http.get_current_user().await?;
    let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
    let embed = CreateEmbed::new()
        .author(author)
        .color(BOT_COLOR)
        .description(include_str!(r"..\include\data.txt"))
        .title("Data Usage and Privacy");

    cmd.create_response(
        ctx,
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .embed(embed)
                .ephemeral(true),
        ),
    )
    .await
    .map_err(Error::from)
}
