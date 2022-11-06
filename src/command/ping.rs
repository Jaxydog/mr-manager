use serenity::{
    async_trait,
    builder::{CreateApplicationCommand, CreateEmbed},
    model::{
        prelude::interaction::{
            application_command::ApplicationCommandInteraction, InteractionResponseType,
        },
        Permissions,
    },
    prelude::Context,
};

use crate::{event::Handler, utility::Result, DEFAULT_COLOR};

use super::SlashCommand;

pub struct Ping;

#[async_trait]
impl SlashCommand for Ping {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("ping")
            .description("Tests the bot's API connection")
            .default_member_permissions(Permissions::USE_SLASH_COMMANDS)
            .dm_permission(true)
    }
    async fn run(
        _: &Handler,
        ctx: &Context,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<()> {
        let mut embed = CreateEmbed::default();
        embed.color(DEFAULT_COLOR).title("Calculating...");

        interaction
            .create_interaction_response(&ctx.http, |res| {
                res.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|msg| msg.add_embed(embed.clone()).ephemeral(true))
            })
            .await?;

        let response = interaction.get_interaction_response(&ctx.http).await?;
        let sent = response.id.created_at().timestamp_millis();
        let created = interaction.id.created_at().timestamp_millis();
        let delay = sent - created;

        embed.title(format!("Pong! ({delay}ms)"));

        interaction
            .edit_original_interaction_response(&ctx.http, |edit| edit.set_embed(embed))
            .await?;

        Ok(())
    }
}
