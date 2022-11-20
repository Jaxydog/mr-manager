use serenity::{
    builder::{
        CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedAuthor,
        CreateInteractionResponse, CreateInteractionResponseMessage,
    },
    model::{
        prelude::{command::CommandOptionType, CommandInteraction},
        Permissions,
    },
    prelude::Context,
};

use crate::{
    utility::{to_unix_str, Result},
    DEFAULT_COLOR,
};

use super::{get_i64, get_str};

pub const NAME: &str = "offer";
pub const GIVE_NAME: &str = "give";
pub const WANT_NAME: &str = "want";
pub const TIME_NAME: &str = "time";

pub fn register() -> CreateCommand {
    CreateCommand::new(NAME)
        .description("Creates a trade offer")
        .default_member_permissions(Permissions::SEND_MESSAGES)
        .dm_permission(false)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                GIVE_NAME,
                "What are you giving away?",
            )
            .max_length(256)
            .clone()
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                WANT_NAME,
                "What do you want in return?",
            )
            .max_length(256)
            .clone()
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Integer,
                TIME_NAME,
                "For how long is this offer valid (in minutes)?",
            )
            .max_int_value(14400)
            .min_int_value(5)
            .required(true),
        )
}

pub async fn run(ctx: &Context, cmd: &CommandInteraction) -> Result<()> {
    let opts = &cmd.data.options();

    let give = get_str(opts, GIVE_NAME)?;
    let want = get_str(opts, WANT_NAME)?;
    let mins = get_i64(opts, TIME_NAME)?;
    let time = to_unix_str(mins * 60 * 1000, "R");

    let embed = CreateEmbed::new()
        .author(CreateEmbedAuthor::new(cmd.user.tag()).icon_url(cmd.user.face()))
        .color(cmd.user.accent_colour.unwrap_or(DEFAULT_COLOR))
        .description(format!("**Offer expires:** {time}"))
        .field("Offer", give, false)
        .field("Price", want, false)
        .thumbnail(cmd.user.face());

    cmd.create_response(
        &ctx.http,
        CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed)),
    )
    .await?;

    Ok(())
}
