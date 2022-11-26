use crate::prelude::*;

pub struct Offer;

pub const NAME: &str = "offer";
const OFFER: &str = "offer";
const PRICE: &str = "price";
const MINUTES: &str = "minutes";

pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .default_member_permissions(Permissions::SEND_MESSAGES)
        .description("Create a trade offer")
        .dm_permission(false)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                OFFER,
                "What are you giving away?",
            )
            .max_length(256)
            .clone()
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                PRICE,
                "What do you want in return?",
            )
            .max_length(256)
            .clone()
            .required(true),
        )
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::Integer,
                MINUTES,
                "For how long is this valid?",
            )
            .max_int_value(14400)
            .min_int_value(5)
            .required(true),
        )
}

pub async fn run_command(ctx: &Context, cmd: &CommandInteraction) -> Result<()> {
    let c = &cmd.data.name;
    let o = &cmd.data.options();

    let user = ctx.http().get_user(cmd.user.id).await?;
    let offer = get_str(c, o, OFFER)?;
    let price = get_str(c, o, PRICE)?;
    let minutes = get_i64(c, o, MINUTES)?;

    let time = timestamp_str(minutes * 60 * 1000, "R");
    let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
    let embed = CreateEmbed::new()
        .author(author)
        .color(user.accent_colour.unwrap_or(BOT_COLOR))
        .description(format!("**Offer expires:** {time}"))
        .field("Offer", offer, false)
        .field("Price", price, false)
        .thumbnail(user.face());

    let message = CreateInteractionResponseMessage::new().embed(embed);
    cmd.create_response(ctx, CreateInteractionResponse::Message(message))
        .await
        .map_err(Error::from)
}
