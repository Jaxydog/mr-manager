use serenity::{
    builder::{
        CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter,
        CreateInteractionResponse, CreateInteractionResponseMessage,
    },
    model::{
        prelude::{command::CommandOptionType, CommandInteraction},
        Color, Permissions,
    },
    prelude::Context,
};

use crate::{utility::Result, DEFAULT_COLOR};

use super::{get_bool, get_str};

pub const NAME: &str = "embed";
pub const AUTHOR_ICON_NAME: &str = "author_icon";
pub const AUTHOR_NAME_NAME: &str = "author_name";
pub const AUTHOR_URL_NAME: &str = "author_url";
pub const COLOR_NAME: &str = "color";
pub const DESCRIPTION_NAME: &str = "description";
pub const FOOTER_ICON_NAME: &str = "footer_icon";
pub const FOOTER_TEXT_NAME: &str = "footer_text";
pub const IMAGE_NAME: &str = "image";
pub const THUMBNAIL_NAME: &str = "thumbnail";
pub const TITLE_NAME: &str = "title";
pub const URL_NAME: &str = "url";
pub const EPHEMERAL_NAME: &str = "ephemeral";

pub fn register() -> CreateCommand {
    CreateCommand::new(NAME)
        .description("Creates a message embed")
        .default_member_permissions(Permissions::EMBED_LINKS.union(Permissions::SEND_MESSAGES))
        .dm_permission(false)
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            AUTHOR_ICON_NAME,
            "The embed author's icon URL",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                AUTHOR_NAME_NAME,
                "The embed author's name",
            )
            .max_length(256)
            .clone(),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            AUTHOR_URL_NAME,
            "The embed author's URL",
        ))
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, COLOR_NAME, "The embed's color")
                .add_string_choice("Default", DEFAULT_COLOR.hex())
                .add_string_choice("Red", Color::RED.hex())
                .add_string_choice("Orange", Color::ORANGE.hex())
                .add_string_choice("Yellow", Color::GOLD.hex())
                .add_string_choice("Green", Color::KERBAL.hex())
                .add_string_choice("Teal", Color::BLITZ_BLUE.hex())
                .add_string_choice("Blue", Color::BLUE.hex())
                .add_string_choice("Purple", Color::PURPLE.hex())
                .add_string_choice("Pink", Color::FABLED_PINK.hex())
                .add_string_choice("Dark Red", Color::DARK_RED.hex())
                .add_string_choice("Dark Orange", Color::DARK_ORANGE.hex())
                .add_string_choice("Dark Yellow", Color::DARK_GOLD.hex())
                .add_string_choice("Dark Green", Color::DARK_GREEN.hex())
                .add_string_choice("Dark Teal", Color::DARK_TEAL.hex())
                .add_string_choice("Dark Blue", Color::DARK_BLUE.hex())
                .add_string_choice("Dark Purple", Color::DARK_PURPLE.hex())
                .add_string_choice("Dark Pink", Color::MEIBE_PINK.hex())
                .add_string_choice("White", Color::LIGHTER_GREY.hex())
                .add_string_choice("Gray", Color::LIGHT_GREY.hex())
                .add_string_choice("Dark Gray", Color::DARK_GREY.hex())
                .add_string_choice("Black", Color::DARKER_GREY.hex()),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                DESCRIPTION_NAME,
                "The embed's description",
            )
            .max_length(4096)
            .clone(),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            FOOTER_ICON_NAME,
            "The embed footer's icon URL",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                FOOTER_TEXT_NAME,
                "The embed footer's text",
            )
            .max_length(2048)
            .clone(),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            IMAGE_NAME,
            "The embed's image URL",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            THUMBNAIL_NAME,
            "The embed's thumbnail URL",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                TITLE_NAME,
                "The embed footer's text",
            )
            .max_length(256)
            .clone(),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            URL_NAME,
            "The embed's URL",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            EPHEMERAL_NAME,
            "Whether to make the embed ephemeral",
        ))
}

pub async fn run(ctx: &Context, cmd: &CommandInteraction) -> Result<()> {
    let opts = &cmd.data.options();
    let mut embed = CreateEmbed::new();
    let mut valid = false;

    if let Ok(name) = get_str(opts, AUTHOR_NAME_NAME) {
        let mut author = CreateEmbedAuthor::new(name);

        if let Ok(icon_url) = get_str(opts, AUTHOR_ICON_NAME) {
            author = author.icon_url(icon_url);
        }
        if let Ok(url) = get_str(opts, AUTHOR_URL_NAME) {
            author = author.url(url);
        }

        embed = embed.author(author);
        valid = true;
    }

    if let Ok(hex) = get_str(opts, COLOR_NAME) {
        let color = u32::from_str_radix(hex, 16).ok().map(Color::new);
        embed = embed.color(color.unwrap_or(DEFAULT_COLOR));
    }

    if let Ok(description) = get_str(opts, DESCRIPTION_NAME) {
        embed = embed.description(description);
        valid = true;
    }

    if let Ok(text) = get_str(opts, FOOTER_TEXT_NAME) {
        let mut footer = CreateEmbedFooter::new(text);

        if let Ok(icon_url) = get_str(opts, FOOTER_ICON_NAME) {
            footer = footer.icon_url(icon_url);
        }

        embed = embed.footer(footer);
        valid = true;
    }

    if let Ok(url) = get_str(opts, IMAGE_NAME) {
        embed = embed.image(url);
        valid = true;
    }

    if let Ok(url) = get_str(opts, THUMBNAIL_NAME) {
        embed = embed.thumbnail(url);
        valid = true;
    }

    if let Ok(title) = get_str(opts, TITLE_NAME) {
        embed = embed.title(title);

        if let Ok(url) = get_str(opts, URL_NAME) {
            embed = embed.url(url);
        }

        valid = true;
    }

    if !valid {
        embed = CreateEmbed::new()
            .color(DEFAULT_COLOR)
            .title("Invalid parameters!");
    }

    let ephemeral = !valid || get_bool(opts, EPHEMERAL_NAME).unwrap_or_default();

    cmd.create_response(
        &ctx.http,
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .embed(embed)
                .ephemeral(ephemeral),
        ),
    )
    .await?;

    Ok(())
}
