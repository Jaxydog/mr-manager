use rand::{thread_rng, Rng};
use serenity::{
    builder::{
        CreateCommand, CreateCommandOption, CreateEmbed, CreateEmbedAuthor,
        CreateInteractionResponse, CreateInteractionResponseMessage,
    },
    model::{
        prelude::{command::CommandOptionType, CommandInteraction},
        Color, Permissions,
    },
    prelude::{CacheHttp, Context},
};

use crate::utility::Result;

use super::get_str;

pub const NAME: &str = "oracle";
pub const QUERY_NAME: &str = "query";

const ANSWERS: [Answer<'static>; 20] = [
    Answer(Mood::Positive, "It is certain."),
    Answer(Mood::Positive, "It is decidedly so."),
    Answer(Mood::Positive, "Without a doubt."),
    Answer(Mood::Positive, "Yes, definitely."),
    Answer(Mood::Positive, "You may rely on it."),
    Answer(Mood::Positive, "As I see it, yes."),
    Answer(Mood::Positive, "Most likely."),
    Answer(Mood::Positive, "Outlook good."),
    Answer(Mood::Positive, "Yes."),
    Answer(Mood::Positive, "Signs point to yes."),
    Answer(Mood::Neutral, "Reply hazy, try again."),
    Answer(Mood::Neutral, "Ask again later."),
    Answer(Mood::Neutral, "Better not tell you now."),
    Answer(Mood::Neutral, "Cannot predict now."),
    Answer(Mood::Neutral, "Concentrate and ask again."),
    Answer(Mood::Negative, "Don't count on it."),
    Answer(Mood::Negative, "My reply is no."),
    Answer(Mood::Negative, "My sources say no."),
    Answer(Mood::Negative, "Outlook not so good."),
    Answer(Mood::Negative, "Very doubtful."),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mood {
    Negative,
    Neutral,
    Positive,
}

impl Mood {
    const fn get_color(self) -> Color {
        match self {
            Self::Negative => Color::RED,
            Self::Neutral => Color::GOLD,
            Self::Positive => Color::KERBAL,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Answer<'a>(Mood, &'a str);

pub fn register() -> CreateCommand {
    CreateCommand::new(NAME)
        .description("Asks the oracle a question")
        .default_member_permissions(Permissions::USE_APPLICATION_COMMANDS)
        .dm_permission(false)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                QUERY_NAME,
                "What would you like to ask?",
            )
            .max_length(512)
            .clone()
            .required(true),
        )
}

pub async fn run(ctx: &Context, cmd: &CommandInteraction) -> Result<()> {
    let name = cmd.user.id.to_user(ctx.http()).await?.tag();
    let opts = &cmd.data.options();
    let query = get_str(opts, QUERY_NAME)?;
    let index = thread_rng().gen_range(0..ANSWERS.len());
    let reply = ANSWERS[index];

    let embed = CreateEmbed::new().author(CreateEmbedAuthor::new("The Oracle").icon_url(
        "https://cdn.discordapp.com/attachments/730389830877577267/1044068278479355954/image.png",
    )).color(reply.0.get_color()).description(format!("{name} asked...\n> {query}\n\n*{}*", reply.1));

    cmd.create_response(
        ctx.http(),
        CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed)),
    )
    .await?;

    Ok(())
}
