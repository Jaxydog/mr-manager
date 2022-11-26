use crate::prelude::*;

pub const NAME: &str = "embed";
pub const AUTHOR_ICON: &str = "author_icon";
pub const AUTHOR_LINK: &str = "author_link";
pub const AUTHOR_NAME: &str = "author_name";
pub const COLOR: &str = "color";
pub const DESCRIPTION: &str = "description";
pub const FOOTER_ICON: &str = "footer_icon";
pub const FOOTER_TEXT: &str = "footer_text";
pub const IMAGE: &str = "image";
pub const THUMBNAIL: &str = "thumbnail";
pub const TITLE_TEXT: &str = "title_text";
pub const TITLE_LINK: &str = "title_link";
pub const EPHEMERAL: &str = "ephemeral";

pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .description("Creates an embedded message")
        .default_member_permissions(Permissions::EMBED_LINKS)
        .dm_permission(false)
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            AUTHOR_ICON,
            "The embed author's icon link",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            AUTHOR_LINK,
            "The embed author's link",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                AUTHOR_NAME,
                "The embed author's name",
            )
            .max_length(256)
            .clone(),
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, COLOR, "The embed's color")
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
                DESCRIPTION,
                "The embed's description (supports newline and markdown)",
            )
            .max_length(4096)
            .clone(),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            FOOTER_ICON,
            "The embed footer's icon link",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                FOOTER_TEXT,
                "The embed footer's text",
            )
            .max_length(2048)
            .clone(),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            IMAGE,
            "The embed's image link",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            THUMBNAIL,
            "The embed's thumbnail link",
        ))
        .add_option(CreateCommandOption::new(
            CommandOptionType::String,
            TITLE_LINK,
            "The embed title's link",
        ))
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                TITLE_TEXT,
                "The embed title's text",
            )
            .max_length(2048)
            .clone(),
        )
        .add_option(CreateCommandOption::new(
            CommandOptionType::Boolean,
            EPHEMERAL,
            "Whether the embed is ephemeral (only visible to you)",
        ))
}

pub async fn run_command(ctx: &Context, cmd: &CommandInteraction) -> Result<()> {
    let c = &cmd.data.name;
    let o = &cmd.data.options();

    let mut embed = CreateEmbed::new();
    let mut length = 0;
    let mut valid = false;

    if let Ok(name) = get_str(c, o, AUTHOR_NAME) {
        let mut author = CreateEmbedAuthor::new(name);

        if let Ok(icon_url) = get_str(c, o, AUTHOR_ICON) {
            author = author.icon_url(icon_url);
        }
        if let Ok(url) = get_str(c, o, AUTHOR_LINK) {
            author = author.url(url);
        }

        embed = embed.author(author);
        length += name.chars().count();
        valid = true;
    }

    if let Ok(hex) = get_str(c, o, COLOR) {
        let color = if hex.is_empty() {
            let user = ctx.http.get_user(cmd.user.id).await?;
            user.accent_colour
        } else {
            u32::from_str_radix(hex, 16).ok().map(Color::new)
        };

        embed = embed.color(color.unwrap_or(BOT_COLOR));
    }

    if let Ok(description) = get_str(c, o, DESCRIPTION) {
        embed = embed.description(description);
        length += description.chars().count();
        valid = true;
    }

    if let Ok(text) = get_str(c, o, FOOTER_TEXT) {
        let mut footer = CreateEmbedFooter::new(text);

        if let Ok(icon_url) = get_str(c, o, FOOTER_ICON) {
            footer = footer.icon_url(icon_url);
        }

        embed = embed.footer(footer);
        length += text.chars().count();
        valid = true;
    }

    if let Ok(url) = get_str(c, o, IMAGE) {
        embed = embed.image(url);
        valid = true;
    }

    if let Ok(url) = get_str(c, o, THUMBNAIL) {
        embed = embed.thumbnail(url);
        valid = true;
    }

    if let Ok(title) = get_str(c, o, TITLE_TEXT) {
        if let Ok(url) = get_str(c, o, TITLE_LINK) {
            embed = embed.url(url);
        }

        embed = embed.title(title);
        length += title.chars().count();
        valid = true;
    }

    if !valid {
        embed = CreateEmbed::new()
            .color(BOT_COLOR)
            .description("At least one visible element must be provided")
            .title("Invalid arguments!");
    } else if length > 6000 {
        embed = CreateEmbed::new()
            .color(BOT_COLOR)
            .description("A maximum of 6000 total characters is allowed")
            .title("Invalid character count!");
    }

    let ephemeral = !valid || length > 6000 || get_bool(c, o, EPHEMERAL).unwrap_or_default();

    cmd.create_response(
        ctx,
        CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .embed(embed)
                .ephemeral(ephemeral),
        ),
    )
    .await
    .map_err(Error::from)
}
