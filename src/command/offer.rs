use serenity::{
    async_trait,
    builder::{CreateApplicationCommand, CreateEmbed},
    model::{
        prelude::{
            command::CommandOptionType,
            interaction::{
                application_command::{ApplicationCommandInteraction, CommandDataOptionValue},
                InteractionResponseType,
            },
        },
        Permissions,
    },
    prelude::Context,
};

use crate::{
    event::Handler,
    utility::{to_unix_str, Error, Result},
    DEFAULT_COLOR,
};

use super::{get_required_data, SlashCommand};

pub struct Offer;

#[async_trait]
impl SlashCommand for Offer {
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("offer")
            .description("Creates a trade offer")
            .default_member_permissions(Permissions::SEND_MESSAGES)
            .dm_permission(false)
            .create_option(|option| {
                option
                    .name("give")
                    .description("What are you giving away?")
                    .kind(CommandOptionType::String)
                    .max_length(256)
                    .min_length(1)
                    .required(true)
            })
            .create_option(|option| {
                option
                    .name("want")
                    .description("What do you want in return?")
                    .kind(CommandOptionType::String)
                    .max_length(256)
                    .min_length(1)
                    .required(true)
            })
            .create_option(|option| {
                option
                    .name("time")
                    .description("For how long is this offer valid (minutes)?")
                    .kind(CommandOptionType::Integer)
                    .max_int_value(14400)
                    .min_int_value(5)
                    .required(true)
            })
    }
    async fn run(
        _: &Handler,
        ctx: &Context,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<()> {
        let give = match get_required_data(interaction, 0)? {
            CommandDataOptionValue::String(s) => Ok(s),
            _ => Err(Error::InvalidCommandData),
        }?;
        let want = match get_required_data(interaction, 1)? {
            CommandDataOptionValue::String(s) => Ok(s),
            _ => Err(Error::InvalidCommandData),
        }?;
        let time = *match get_required_data(interaction, 2)? {
            CommandDataOptionValue::Integer(i) => Ok(i),
            _ => Err(Error::InvalidCommandData),
        }?;
        let time_str = to_unix_str(time * 60 * 1000, "R");

        let mut embed = CreateEmbed::default();
        embed
            .author(|author| {
                author
                    .icon_url(interaction.user.face())
                    .name(interaction.user.tag())
            })
            .color(DEFAULT_COLOR)
            .description(format!("**Offer expires:** {time_str}"))
            .field("Offer", give, false)
            .field("Price", want, false)
            .thumbnail(interaction.user.face());

        interaction
            .create_interaction_response(&ctx.http, |res| {
                res.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| data.add_embed(embed))
            })
            .await?;

        Ok(())
    }
}
