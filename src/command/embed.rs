use crate::prelude::*;

pub const NAME: &str = "embed";

pub const OPTION_AUTHOR_ICON: &str = "author_icon";
pub const OPTION_AUTHOR_LINK: &str = "author_link";
pub const OPTION_AUTHOR_NAME: &str = "author_name";
pub const OPTION_EMBED_COLOR: &str = "color";
pub const OPTION_DESCRIPTION: &str = "description";
pub const OPTION_FOOTER_ICON: &str = "footer_icon";
pub const OPTION_FOOTER_TEXT: &str = "footer_text";
pub const OPTION_IMAGE_LINK: &str = "image_link";
pub const OPTION_THUMB_LINK: &str = "thumbnail_link";
pub const OPTION_TITLE_TEXT: &str = "title_text";
pub const OPTION_TITLE_LINK: &str = "title_link";
pub const OPTION_EPHEMERAL: &str = "ephemeral";

#[allow(clippy::too_many_lines)]
pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .default_member_permissions(Permissions::EMBED_LINKS)
        .description("Creates an embedded message")
        .dm_permission(false)
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            OPTION_AUTHOR_ICON,
            "The embed author's icon link",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            OPTION_AUTHOR_LINK,
            "The embed author's link",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                OPTION_AUTHOR_NAME,
                "The embed author's name",
            )
            .max_length(256)
            .clone(),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                OPTION_EMBED_COLOR,
                "The embed's color",
            )
            .add_string_choice("Default", BOT_COLOR.hex())
            .add_string_choice("User", String::new())
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
                OPTION_DESCRIPTION,
                "The embed's description (supports newline and markdown)",
            )
            .max_length(4096)
            .clone(),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            OPTION_FOOTER_ICON,
            "The embed footer's icon link",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                OPTION_FOOTER_TEXT,
                "The embed footer's text",
            )
            .max_length(2048)
            .clone(),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            OPTION_IMAGE_LINK,
            "The embed's image link",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            OPTION_THUMB_LINK,
            "The embed's thumbnail link",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            OPTION_TITLE_LINK,
            "The embed title's link",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                OPTION_TITLE_TEXT,
                "The embed title's text",
            )
            .max_length(2048)
            .clone(),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::Boolean,
            OPTION_EPHEMERAL,
            "Whether the embed is ephemeral (only visible to you)",
        ))
}

pub async fn run_command(http: &Http, cmd: &CommandInteraction) -> Result<()> {
    let o = &cmd.data.options();
    let mut embed = CreateEmbed::new();
    let mut count = 0;
    let mut valid = false;

    if let Ok(name) = get_str(o, OPTION_AUTHOR_NAME) {
        let mut author = CreateEmbedAuthor::new(name);

        if let Ok(icon_url) = get_str(o, OPTION_AUTHOR_ICON) {
            author = author.icon_url(icon_url);
        }
        if let Ok(url) = get_str(o, OPTION_AUTHOR_LINK) {
            author = author.url(url);
        }

        embed = embed.author(author);
        count += name.chars().count();
        valid = true;
    }

    if let Ok(hex) = get_str(o, OPTION_EMBED_COLOR) {
        let color = if hex.is_empty() {
            let user = http.get_user(cmd.user.id).await?;

            user.accent_colour
        } else {
            u32::from_str_radix(hex, 16).ok().map(Color::new)
        };

        embed = embed.color(color.unwrap_or(BOT_COLOR));
    }

    if let Ok(description) = get_str(o, OPTION_DESCRIPTION) {
        let description = description.replace(r"\n", "\n");

        embed = embed.description(description.trim());
        count += description.trim().chars().count();
        valid = true;
    }

    if let Ok(text) = get_str(o, OPTION_FOOTER_TEXT) {
        let mut footer = CreateEmbedFooter::new(text);

        if let Ok(icon_url) = get_str(o, OPTION_FOOTER_ICON) {
            footer = footer.icon_url(icon_url);
        }

        embed = embed.footer(footer);
        count += text.chars().count();
        valid = true;
    }

    if let Ok(url) = get_str(o, OPTION_IMAGE_LINK) {
        embed = embed.image(url);
        valid = true;
    }

    if let Ok(url) = get_str(o, OPTION_THUMB_LINK) {
        embed = embed.thumbnail(url);
        valid = true;
    }

    if let Ok(title) = get_str(o, OPTION_TITLE_TEXT) {
        if let Ok(url) = get_str(o, OPTION_TITLE_LINK) {
            embed = embed.url(url);
        }

        embed = embed.title(title);
        count += title.chars().count();
        valid = true;
    }

    if !valid {
        return Err(Error::Other("A visible element must be provided"));
    }
    if count > 6000 {
        return Err(Error::Other("Content must have at most 6000 characters"));
    }

    let ephemeral = get_bool(o, OPTION_EPHEMERAL).unwrap_or_default();
    let message = CreateInteractionResponseMessage::new()
        .embed(embed)
        .ephemeral(ephemeral);

    cmd.create_response(http, CreateInteractionResponse::Message(message))
        .await
        .map_err(Error::from)
}
