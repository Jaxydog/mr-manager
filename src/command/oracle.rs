use rand::{thread_rng, Rng};

use crate::prelude::*;

pub const NAME: &str = "oracle";
pub const QUERY: &str = "question";
pub const ORACLE_URL: &str =
    "https://cdn.discordapp.com/attachments/730389830877577267/1044068278479355954/image.png";
pub const ANSWERS: [Answer; 20] = [
    Answer::new(Mood::Positive, "It is certain."),
    Answer::new(Mood::Positive, "It is decidedly so."),
    Answer::new(Mood::Positive, "Without a doubt."),
    Answer::new(Mood::Positive, "Yes, definitely."),
    Answer::new(Mood::Positive, "You may rely on it."),
    Answer::new(Mood::Positive, "As I see it, yes."),
    Answer::new(Mood::Positive, "Most likely."),
    Answer::new(Mood::Positive, "Outlook good."),
    Answer::new(Mood::Positive, "Yes."),
    Answer::new(Mood::Positive, "Signs point to yes."),
    Answer::new(Mood::Neutral, "Reply hazy, try again."),
    Answer::new(Mood::Neutral, "Ask again later."),
    Answer::new(Mood::Neutral, "Better not tell you now."),
    Answer::new(Mood::Neutral, "Cannot predict now."),
    Answer::new(Mood::Neutral, "Concentrate and ask again."),
    Answer::new(Mood::Negative, "Don't count on it."),
    Answer::new(Mood::Negative, "My reply is no."),
    Answer::new(Mood::Negative, "My sources say no."),
    Answer::new(Mood::Negative, "Outlook not so good."),
    Answer::new(Mood::Negative, "Very doubtful."),
];

#[derive(Clone, Copy, Debug)]
pub enum Mood {
    Negative,
    Neutral,
    Positive,
}

#[derive(Clone, Copy, Debug)]
pub struct Answer {
    pub mood: Mood,
    pub text: &'static str,
}

impl Answer {
    #[must_use]
    pub const fn new(mood: Mood, text: &'static str) -> Self {
        Self { mood, text }
    }
}

#[async_trait]
impl ToEmbed for Answer {
    type Args = (String, String);

    async fn to_embed(&self, _: &Context, (name, query): Self::Args) -> Result<CreateEmbed> {
        let author = CreateEmbedAuthor::new("The Oracle").icon_url(ORACLE_URL);
        let description = format!("**{name} asked...**\n> {query}\n\n*{}*", self.text);
        let color = match self.mood {
            Mood::Negative => Color::RED,
            Mood::Neutral => Color::GOLD,
            Mood::Positive => Color::KERBAL,
        };

        Ok(CreateEmbed::new()
            .author(author)
            .color(color)
            .description(description))
    }
}

pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .description("Asks the Oracle a question")
        .default_member_permissions(Permissions::SEND_MESSAGES)
        .dm_permission(false)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                QUERY,
                "What would you like to ask?",
            )
            .max_length(512)
            .clone()
            .required(true),
        )
}

pub async fn run_command(ctx: &Context, cmd: &CommandInteraction) -> Result<()> {
    let name = cmd.user.id.to_user(ctx).await?.tag();
    let o = &cmd.data.options();
    let query = get_str(&cmd.data.name, o, QUERY)?;
    let index = thread_rng().gen_range(0..ANSWERS.len());
    let reply = ANSWERS[index];
    let embed = reply.to_embed(ctx, (name, query.to_string())).await?;

    let message = CreateInteractionResponseMessage::new().embed(embed);
    cmd.create_response(ctx, CreateInteractionResponse::Message(message))
        .await
        .map_err(Error::from)
}
