use crate::prelude::*;

pub const NAME: &str = "help";

pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .description("Displays a list of bot commands")
        .default_member_permissions(Permissions::USE_APPLICATION_COMMANDS)
        .dm_permission(true)
}

pub async fn run_command(ctx: &Context, cmd: &CommandInteraction) -> Result<()> {
    let user = ctx.http.get_current_user().await?;
    let mut commands = ctx.http.get_global_application_commands().await?;
    let mut description = include_str!(r"..\include\help\start.txt").to_string();

    if commands.is_empty() {
        description.push_str("\n*No global commands found...*\n");
    } else {
        commands.sort_by_key(|c| c.name.clone());

        for command in commands {
            let name = command.name;
            let id = command.id;
            let info = command.description;

            description.push_str(&format!("\n</{name}:{id}> - {info}"));
        }

        description.push('\n');
    }

    description.push_str(include_str!(r"..\include\help\end.txt"));

    let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
    let embed = CreateEmbed::new()
        .author(author)
        .color(BOT_COLOR)
        .description(description)
        .title("Command Menu");

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
