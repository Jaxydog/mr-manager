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
    utils::Color,
};

use crate::{event::Handler, utility::Result, DEFAULT_COLOR};

use super::{get_data, SlashCommand};

pub struct Embed;

#[async_trait]
impl SlashCommand for Embed {
    #[allow(clippy::too_many_lines)]
    fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
        command
            .name("embed")
            .description("Creates a rich message embed")
            .default_member_permissions(Permissions::EMBED_LINKS)
            .dm_permission(false)
            .create_option(|option| {
                option
                    .name("author_icon")
                    .description("The embed author's icon URL")
                    .kind(CommandOptionType::String)
            })
            .create_option(|option| {
                option
                    .name("author_name")
                    .description("The embed author's name")
                    .kind(CommandOptionType::String)
                    .max_length(256)
            })
            .create_option(|option| {
                option
                    .name("author_url")
                    .description("The embed author's URL")
                    .kind(CommandOptionType::String)
            })
            .create_option(|option| {
                option
                    .name("color")
                    .description("The embed's color")
                    .kind(CommandOptionType::String)
                    .add_string_choice("Default", DEFAULT_COLOR.hex())
                    .add_string_choice("Red", Color::RED.hex())
                    .add_string_choice("Orange", Color::ORANGE.hex())
                    .add_string_choice("Yellow", Color::GOLD.hex())
                    .add_string_choice("Green", Color::KERBAL.hex())
                    .add_string_choice("Aqua", Color::BLITZ_BLUE.hex())
                    .add_string_choice("Blue", Color::BLUE.hex())
                    .add_string_choice("Purple", Color::PURPLE.hex())
                    .add_string_choice("Pink", Color::FABLED_PINK.hex())
                    .add_string_choice("Dark Red", Color::DARK_RED.hex())
                    .add_string_choice("Dark Orange", Color::DARK_ORANGE.hex())
                    .add_string_choice("Dark Yellow", Color::DARK_GOLD.hex())
                    .add_string_choice("Dark Green", Color::DARK_GREEN.hex())
                    .add_string_choice("Dark Aqua", Color::DARK_TEAL.hex())
                    .add_string_choice("Dark Blue", Color::DARK_BLUE.hex())
                    .add_string_choice("Dark Purple", Color::DARK_PURPLE.hex())
                    .add_string_choice("Dark Pink", Color::MEIBE_PINK.hex())
                    .add_string_choice("White", Color::LIGHTER_GREY.hex())
                    .add_string_choice("Gray", Color::LIGHT_GREY.hex())
                    .add_string_choice("Dark Gray", Color::DARK_GREY.hex())
                    .add_string_choice("Black", Color::DARKER_GREY.hex())
            })
            .create_option(|option| {
                option
                    .name("description")
                    .description("The embed's description")
                    .kind(CommandOptionType::String)
                    .max_length(4096)
            })
            .create_option(|option| {
                option
                    .name("footer_icon")
                    .description("The embed footer's icon URL")
                    .kind(CommandOptionType::String)
            })
            .create_option(|option| {
                option
                    .name("footer_text")
                    .description("The embed footer's text")
                    .kind(CommandOptionType::String)
                    .max_length(2048)
            })
            .create_option(|option| {
                option
                    .name("image")
                    .description("The embed's image URL")
                    .kind(CommandOptionType::String)
            })
            .create_option(|option| {
                option
                    .name("thumbnail")
                    .description("The embed's thumbnail URL")
                    .kind(CommandOptionType::String)
            })
            .create_option(|option| {
                option
                    .name("title")
                    .description("The embed's title")
                    .kind(CommandOptionType::String)
                    .max_length(256)
            })
            .create_option(|option| {
                option
                    .name("url")
                    .description("The embed's URL")
                    .kind(CommandOptionType::String)
            })
            .create_option(|option| {
                option
                    .name("ephemeral")
                    .description("Whether to make the embed ephemeral")
                    .kind(CommandOptionType::Boolean)
            })
    }
    async fn run(
        _: &Handler,
        ctx: &Context,
        interaction: &ApplicationCommandInteraction,
    ) -> Result<()> {
        let mut embed = CreateEmbed::default();

        if let Some(CommandDataOptionValue::String(name)) = get_data(interaction, 1) {
            embed.author(|author| {
                if let Some(CommandDataOptionValue::String(icon_url)) = get_data(interaction, 0) {
                    author.icon_url(icon_url);
                }
                if let Some(CommandDataOptionValue::String(url)) = get_data(interaction, 2) {
                    author.url(url);
                }
                author.name(name)
            });
        }

        if let Some(CommandDataOptionValue::String(hex)) = get_data(interaction, 3) {
            if let Ok(value) = hex.parse::<u64>() {
                embed.color(value);
            }
        }

        if let Some(CommandDataOptionValue::String(description)) = get_data(interaction, 4) {
            embed.description(description);
        }

        if let Some(CommandDataOptionValue::String(text)) = get_data(interaction, 6) {
            embed.footer(|footer| {
                if let Some(CommandDataOptionValue::String(icon_url)) = get_data(interaction, 5) {
                    footer.icon_url(icon_url);
                }

                footer.text(text)
            });
        }

        if let Some(CommandDataOptionValue::String(image)) = get_data(interaction, 7) {
            embed.image(image);
        }

        if let Some(CommandDataOptionValue::String(thumbnail)) = get_data(interaction, 8) {
            embed.thumbnail(thumbnail);
        }

        if let Some(CommandDataOptionValue::String(title)) = get_data(interaction, 9) {
            embed.title(title);
        }

        if let Some(CommandDataOptionValue::String(url)) = get_data(interaction, 10) {
            embed.url(url);
        }

        let ephemeral = match get_data(interaction, 11) {
            Some(CommandDataOptionValue::Boolean(b)) => *b,
            _ => false,
        };

        interaction
            .create_interaction_response(&ctx.http, |res| {
                res.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|data| data.add_embed(embed).ephemeral(ephemeral))
            })
            .await?;

        Ok(())
    }
}
