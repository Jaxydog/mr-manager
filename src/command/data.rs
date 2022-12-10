use crate::prelude::*;

pub const NAME: &str = "data";

pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .default_member_permissions(Permissions::USE_APPLICATION_COMMANDS)
        .description("Displays information about data usage and privacy")
        .dm_permission(true)
}

pub async fn run_command(http: &Http, cmd: &CommandInteraction) -> Result<()> {
    let user = http.get_current_user().await?;
    let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
    let embed = CreateEmbed::new()
        .author(author)
        .color(BOT_COLOR)
        .description(include_str!(r"..\include\data.txt"))
        .title("Data Usage and Privacy");
    let message = CreateInteractionResponseMessage::new()
        .embed(embed)
        .ephemeral(true);

    cmd.create_response(http, CreateInteractionResponse::Message(message))
        .await
        .map_err(Error::from)
}
