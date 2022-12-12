use crate::prelude::*;

pub const NAME: &str = "offer";

pub const OPTION_OFFER: &str = "offer";
pub const OPTION_PRICE: &str = "price";
pub const OPTION_MINUTES: &str = "minutes";

pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .default_member_permissions(Permissions::SEND_MESSAGES)
        .description("Create a new trade offer")
        .dm_permission(false)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                OPTION_OFFER,
                "What are you giving away?",
            )
            .max_length(256)
            .clone()
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                OPTION_PRICE,
                "What do you want in return?",
            )
            .max_length(256)
            .clone()
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Integer,
                OPTION_MINUTES,
                "For how long is this valid?",
            )
            .max_int_value(14400)
            .min_int_value(5)
            .required(true),
        )
}

pub async fn run_command(http: &Http, cmd: &CommandInteraction) -> Result<()> {
    let user = http.get_user(cmd.user.id).await?;

    let o = &cmd.data.options();
    let offer = get_str(o, OPTION_OFFER)?;
    let price = get_str(o, OPTION_PRICE)?;
    let minutes = get_i64(o, OPTION_MINUTES)?;

    let time = TimeString::new(Utc::now().timestamp_millis() + (minutes * 60 * 1000));
    let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
    let embed = CreateEmbed::new()
        .author(author)
        .color(user.accent_colour.unwrap_or(BOT_COLOR))
        .description(format!("**Expires:** {time}"))
        .field("Offer", offer, false)
        .field("Price", price, false)
        .thumbnail(user.face());

    let message = CreateInteractionResponseMessage::new().embed(embed);
    cmd.create_response(http, CreateInteractionResponse::Message(message))
        .await
        .map_err(Error::from)
}
