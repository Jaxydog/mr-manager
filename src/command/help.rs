use serenity::{
    builder::{
        CreateCommand, CreateEmbed, CreateEmbedAuthor, CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
    model::{
        prelude::{command::Command, CommandInteraction},
        Permissions,
    },
    prelude::Context,
};

use crate::{utility::Result, DEFAULT_COLOR};

pub const NAME: &str = "help";

pub fn register() -> CreateCommand {
    CreateCommand::new(NAME)
        .description("Displays information about the bot")
        .default_member_permissions(Permissions::USE_APPLICATION_COMMANDS)
        .dm_permission(true)
}

fn stringify(cmd: &Command) -> String {
    format!("\n</{}:{}> - {}", cmd.name, cmd.id, cmd.description)
}

pub async fn run(ctx: &Context, cmd: &CommandInteraction) -> Result<()> {
    let user = ctx.http.get_current_user().await?;
    let mut commands = ctx.http.get_global_application_commands().await?;

    let mut description = include_str!(r"..\include\help\start.txt").to_string();

    if commands.is_empty() {
        description.push_str("\n*No global commands found...*\n");
    } else {
        commands.sort_by_key(|c| c.name.clone());

        for listing in commands.iter().map(stringify) {
            description.push_str(&listing);
        }
    }

    description.push_str(include_str!(r"..\include\help\end.txt"));

    let embed = CreateEmbed::new()
        .author(CreateEmbedAuthor::new(user.tag()).icon_url(user.face()))
        .color(DEFAULT_COLOR)
        .description(description)
        .title("Command Menu");

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
