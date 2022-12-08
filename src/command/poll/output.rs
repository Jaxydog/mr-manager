use std::collections::BTreeMap;

use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Output {
    MultipleChoice(MultipleChoiceOutput),
    RandomRaffle(RandomRaffleOutput),
    TextResponse(TextResponseOutput),
}

impl Output {
    pub const fn kind(&self) -> Kind {
        match self {
            Self::MultipleChoice(_) => Kind::MultipleChoice,
            Self::RandomRaffle(_) => Kind::RandomRaffle,
            Self::TextResponse(_) => Kind::TextResponse,
        }
    }
    pub fn total(&self) -> usize {
        match self {
            Self::MultipleChoice(i) => i.total,
            Self::RandomRaffle(i) => i.users.len(),
            Self::TextResponse(i) => i.total,
        }
    }
    pub fn pages(&self) -> usize {
        match self {
            Self::MultipleChoice(o) => o.pages(),
            Self::RandomRaffle(o) => o.pages(),
            Self::TextResponse(o) => o.pages(),
        }
    }
}

#[async_trait]
impl TryAsEmbedAsync for Output {
    type Args<'a> = (&'a Poll, usize);

    async fn try_as_embed(
        &self,
        ctx: &Context,
        (poll, page): Self::Args<'_>,
    ) -> Result<CreateEmbed> {
        let pages = self.pages();

        let page = if page == 0 {
            pages
        } else if page > pages {
            1
        } else {
            pages
        };

        match self {
            Self::MultipleChoice(o) => o.try_as_embed(ctx, (poll, page)).await,
            Self::RandomRaffle(o) => o.try_as_embed(ctx, (poll, page)).await,
            Self::TextResponse(o) => o.try_as_embed(ctx, (poll, page)).await,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultipleChoiceOutput {
    pub total: usize,
    pub entries: Vec<MultipleChoiceOutputEntry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MultipleChoiceOutputEntry {
    pub votes: usize,
    pub users: Vec<UserId>,
}

impl MultipleChoiceOutput {
    pub const BAR_FILL: char = '█';

    pub fn pages(&self) -> usize {
        self.entries.len() + 1
    }

    #[allow(clippy::cast_precision_loss)]
    fn __percent(votes: usize, total: usize) -> f64 {
        votes as f64 / total as f64
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn __graph_small(percent: f64) -> String {
        let filled = 10 * percent as usize;
        let fill = Self::BAR_FILL.to_string().repeat(filled);

        format!("`{fill: >10}`")
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn __graph_large(percent: f64) -> String {
        let filled = 32 * percent as usize;
        let fill = Self::BAR_FILL.to_string().repeat(filled);

        format!("`{fill: >32}`")
    }

    async fn __overview(&self, ctx: &Context, poll: &Poll) -> Result<CreateEmbed> {
        let user = ctx.http.get_user(poll.user).await?;

        let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
        let footer = CreateEmbedFooter::new(format!("Page: 1 / {}", self.pages()));
        let mut description = format!("**Total Votes:** {}\n\n", self.total);

        for (index, entry) in self.entries.iter().enumerate() {
            let Some(Input::MultipleChoice(input)) = poll.inputs.get(index) else {
				continue;
			};

            let percent = Self::__percent(entry.votes, self.total);
            let graph = Self::__graph_small(percent);
            let votes = format!("{} votes ({:.1}%)", entry.votes, percent * 100.0);

            description.push_str(&format!("{graph} {} - {votes}\n", input.label));
        }

        let embed = CreateEmbed::new()
            .author(author)
            .color(user.accent_colour.unwrap_or(BOT_COLOR))
            .description(description)
            .footer(footer)
            .thumbnail(user.face())
            .title("Poll Results: Overview");

        if let Ok(anchor) = poll.anchor() {
            Ok(embed.url(anchor.to_string()))
        } else {
            Ok(embed)
        }
    }
    async fn __page(&self, ctx: &Context, poll: &Poll, page: usize) -> Result<CreateEmbed> {
        let index = page - 2;

        let Some(entry) = self.entries.get(index) else {
			return Err(Error::MissingValue(Value::Other("Entry")));
		};
        let Some(Input::MultipleChoice(input)) = poll.inputs.get(index) else {
			return Err(Error::MissingValue(Value::Other("Input")));
		};

        let user = ctx.http.get_user(poll.user).await?;

        let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
        let footer = CreateEmbedFooter::new(format!("Page: {page} / {}", self.pages()));

        let percent = Self::__percent(entry.votes, self.total);
        let votes = format!("**Total Votes:** {} ({percent:.1}%)", entry.votes);
        let graph = Self::__graph_large(percent);
        let mut description = format!("{votes}\n{graph}\n\n**Users:**\n");

        if poll.content.hide_members {
            description.push_str("*Users are hidden*");
        } else {
            let users = entry
                .users
                .iter()
                .fold(String::new(), |s, id| format!("{s}<@{id}>\n"));

            description.push_str(&users);
        }

        let embed = CreateEmbed::new()
            .author(author)
            .color(user.accent_colour.unwrap_or(BOT_COLOR))
            .description(description)
            .footer(footer)
            .thumbnail(user.face())
            .title(format!("Poll Results: {}", input.label));

        if let Ok(anchor) = poll.anchor() {
            Ok(embed.url(anchor.to_string()))
        } else {
            Ok(embed)
        }
    }
}

#[async_trait]
impl TryAsEmbedAsync for MultipleChoiceOutput {
    type Args<'a> = (&'a Poll, usize);

    async fn try_as_embed(
        &self,
        ctx: &Context,
        (poll, page): Self::Args<'_>,
    ) -> Result<CreateEmbed> {
        if page == 1 {
            self.__overview(ctx, poll).await
        } else {
            self.__page(ctx, poll, page).await
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RandomRaffleOutput {
    pub winner: UserId,
    pub users: Vec<UserId>,
}

impl RandomRaffleOutput {
    #[allow(clippy::unused_self)]
    pub const fn pages(&self) -> usize {
        1
    }

    async fn __overview(&self, ctx: &Context, poll: &Poll) -> Result<CreateEmbed> {
        let user = ctx.http.get_user(poll.user).await?;

        let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
        let footer = CreateEmbedFooter::new(format!("Page: 1 / {}", self.pages()));

        let total = format!("**Total Entries:** {}", self.users.len());
        let winner = format!("**Winner:** <@{}>", self.winner);
        let mut description = format!("{total}\n{winner}\n\n**Users:**\n");

        if poll.content.hide_members {
            description.push_str("*Users are hidden*");
        } else {
            let users = self
                .users
                .iter()
                .fold(String::new(), |s, id| format!("{s}<@{id}>\n"));

            description.push_str(&users);
        }

        let embed = CreateEmbed::new()
            .author(author)
            .color(user.accent_colour.unwrap_or(BOT_COLOR))
            .description(description)
            .footer(footer)
            .thumbnail(user.face())
            .title("Poll Results: Overview");

        if let Ok(anchor) = poll.anchor() {
            Ok(embed.url(anchor.to_string()))
        } else {
            Ok(embed)
        }
    }
}

#[async_trait]
impl TryAsEmbedAsync for RandomRaffleOutput {
    type Args<'a> = (&'a Poll, usize);

    async fn try_as_embed(&self, ctx: &Context, (poll, _): Self::Args<'_>) -> Result<CreateEmbed> {
        self.__overview(ctx, poll).await
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TextResponseOutput {
    pub total: usize,
    pub answers: BTreeMap<UserId, Vec<String>>,
}

impl TextResponseOutput {
    pub fn pages(&self) -> usize {
        self.answers.len() + 1
    }

    async fn __overview(&self, ctx: &Context, poll: &Poll) -> Result<CreateEmbed> {
        let user = ctx.http.get_user(poll.user).await?;

        let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
        let footer = CreateEmbedFooter::new(format!("Page: 1 / {}", self.pages()));
        let description = format!("**Total Responses:** {}", self.answers.len());

        let embed = CreateEmbed::new()
            .author(author)
            .color(user.accent_colour.unwrap_or(BOT_COLOR))
            .description(description)
            .footer(footer)
            .thumbnail(user.face())
            .title("Poll Results: Overview");

        if let Ok(anchor) = poll.anchor() {
            Ok(embed.url(anchor.to_string()))
        } else {
            Ok(embed)
        }
    }
    async fn __page(&self, ctx: &Context, poll: &Poll, page: usize) -> Result<CreateEmbed> {
        let index = page - 2;

        let Some((user, answers)) = self.answers.iter().nth(index) else {
			return Err(Error::MissingValue(Value::Other("Entry")));
		};

        let user = ctx.http.get_user(poll.user).await?;

        let author = if poll.content.hide_members {
            CreateEmbedAuthor::new("Anonymous User").icon_url(user.default_avatar_url())
        } else {
            CreateEmbedAuthor::new(user.tag()).icon_url(user.face())
        };
        let color = if poll.content.hide_members {
            BOT_COLOR
        } else {
            user.accent_colour.unwrap_or(BOT_COLOR)
        };
        let thumbnail = if poll.content.hide_members {
            user.default_avatar_url()
        } else {
            user.face()
        };
        let title = if poll.content.hide_members {
            format!("Poll Results: User {}", page - 1)
        } else {
            format!("Poll Results: {}", user.name)
        };

        let footer = CreateEmbedFooter::new(format!("Page: {page} / {}", self.pages()));

        let mut embed = CreateEmbed::new()
            .author(author)
            .color(color)
            .footer(footer)
            .thumbnail(thumbnail)
            .title(title);

        for (index, answer) in answers.iter().enumerate() {
            let Some(Input::TextResponse(input)) = poll.inputs.get(index) else {
				continue;
			};

            embed = embed.field(&input.label, answer, false);
        }

        if let Ok(anchor) = poll.anchor() {
            Ok(embed.url(anchor.to_string()))
        } else {
            Ok(embed)
        }
    }
}

#[async_trait]
impl TryAsEmbedAsync for TextResponseOutput {
    type Args<'a> = (&'a Poll, usize);

    async fn try_as_embed(
        &self,
        ctx: &Context,
        (poll, page): Self::Args<'_>,
    ) -> Result<CreateEmbed> {
        if page == 1 {
            self.__overview(ctx, poll).await
        } else {
            self.__page(ctx, poll, page).await
        }
    }
}
