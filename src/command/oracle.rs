use rand::{thread_rng, Rng};

use crate::prelude::*;

pub const NAME: &str = "oracle";

pub const OPTION_QUESTION: &str = "question";

pub const ANSWERS: [Reply; 20] = [
    Reply::new(Mood::Good, "It is certain."),
    Reply::new(Mood::Good, "It is decidedly so."),
    Reply::new(Mood::Good, "Without a doubt."),
    Reply::new(Mood::Good, "Yes, definitely."),
    Reply::new(Mood::Good, "You may rely on it."),
    Reply::new(Mood::Good, "As I see it, yes."),
    Reply::new(Mood::Good, "Most likely."),
    Reply::new(Mood::Good, "Outlook good."),
    Reply::new(Mood::Good, "Yes."),
    Reply::new(Mood::Good, "Signs point to yes."),
    Reply::new(Mood::Unsure, "Reply hazy, try again."),
    Reply::new(Mood::Unsure, "Ask again later."),
    Reply::new(Mood::Unsure, "Better not tell you now."),
    Reply::new(Mood::Unsure, "Cannot predict now."),
    Reply::new(Mood::Unsure, "Concentrate and ask again."),
    Reply::new(Mood::Bad, "Don't count on it."),
    Reply::new(Mood::Bad, "My reply is no."),
    Reply::new(Mood::Bad, "My sources say no."),
    Reply::new(Mood::Bad, "Outlook not so good."),
    Reply::new(Mood::Bad, "Very doubtful."),
];

#[derive(Clone, Copy, Debug)]
pub enum Mood {
    Bad,
    Unsure,
    Good,
}

impl Mood {
    pub const fn color(self) -> Color {
        match self {
            Self::Bad => Color::RED,
            Self::Unsure => Color::LIGHT_GREY,
            Self::Good => Color::KERBAL,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Reply {
    pub mood: Mood,
    pub text: &'static str,
}

impl Reply {
    pub const fn new(mood: Mood, text: &'static str) -> Self {
        Self { mood, text }
    }
}

impl AsEmbed<(&str, &str)> for Reply {
    fn as_embed(&self, (name, question): (&str, &str)) -> CreateEmbed {
        let author = CreateEmbedAuthor::new("The Oracle").icon_url("https://cdn.discordapp.com/attachments/730389830877577267/1044068278479355954/image.png");
        let description = format!("**{name} asked...**\n> {question}\n\n*{}*", self.text);

        CreateEmbed::new()
            .author(author)
            .color(self.mood.color())
            .description(description)
    }
}

pub fn new() -> CreateCommand {
    CreateCommand::new(NAME)
        .default_member_permissions(Permissions::SEND_MESSAGES)
        .description("Asks the Oracle a question")
        .dm_permission(false)
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                OPTION_QUESTION,
                "What would you like to ask?",
            )
            .max_length(512)
            .clone()
            .required(true),
        )
}

pub async fn run_command(http: &Http, cmd: &CommandInteraction) -> Result<()> {
    let name = cmd.user.id.to_user(http).await?.tag();
    let o = &cmd.data.options();
    let query = get_str(o, OPTION_QUESTION)?;

    let index = thread_rng().gen_range(0..ANSWERS.len());
    let reply = ANSWERS[index];
    let embed = reply.as_embed((&name, query));

    let message = CreateInteractionResponseMessage::new().embed(embed);
    cmd.create_response(http, CreateInteractionResponse::Message(message))
        .await
        .map_err(Error::from)
}
