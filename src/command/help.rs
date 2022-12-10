use crate::prelude::*;

pub const NAME: &str = "help";

pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .default_member_permissions(Permissions::USE_APPLICATION_COMMANDS)
        .description("Displays a list of bot commands")
        .dm_permission(false)
}

pub async fn run_command(http: &Http, cmd: &CommandInteraction) -> Result<()> {
    let user = http.get_current_user().await?;
    let mut commands = http.get_global_application_commands().await?;
    let mut description = include_str!(r"..\include\help\start.txt").to_string();

    if commands.is_empty() {
        description.push_str("\n*No global commands found...*\n");
    } else {
        commands.sort_by_key(|c| c.name.clone());

        for command in commands {
            let label = format!(
                "\n</{}:{}> - {}",
                command.name, command.id, command.description
            );

            description.push_str(&label);
        }

        description.push('\n');
    }

    description.push_str(include_str!(r"..\include\help\end.txt"));

    let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
    let embed = CreateEmbed::new()
        .author(author)
        .color(BOT_COLOR)
        .description(description)
        .title("Command List");

    let message = CreateInteractionResponseMessage::new()
        .embed(embed)
        .ephemeral(true);

    cmd.create_response(http, CreateInteractionResponse::Message(message))
        .await
        .map_err(Error::from)
}
