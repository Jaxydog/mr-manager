use std::collections::{BTreeMap, BTreeSet};

use rand::{thread_rng, Rng};

use super::*;

pub const BUTTON_LAST: &str = formatcp!("{NAME}_last");
pub const BUTTON_NEXT: &str = formatcp!("{NAME}_next");

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChoiceOutputData {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub votes: Vec<(usize, BTreeSet<UserId>)>,
}

impl ChoiceOutputData {
    pub const BAR_FILL: &str = "█";
    pub const BAR_NONE: &str = " ";

    pub fn new(form: &Form) -> Self {
        let mut votes = vec![];

        for (index, input) in form.inputs.iter().enumerate() {
            let Input::Choice(_) = input else {
				continue;
			};

            let mut users = BTreeSet::new();

            for (user, reply) in &form.replies {
                let Reply::Choice(data) = reply else {
					continue;
				};
                if index != *data {
                    continue;
                }

                users.insert(*user);
            }

            votes.push((index, users));
        }

        votes.sort_by_key(|(_, users)| users.len());
        votes.reverse();

        Self { votes }
    }

    pub fn pages(&self) -> usize {
        self.votes.len() + 1
    }

    #[allow(clippy::cast_precision_loss)]
    fn __percent(votes: usize, total: usize) -> f64 {
        if total == 0 {
            0.0
        } else {
            votes as f64 / total as f64
        }
    }
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss
    )]
    fn __graph(percent: f64, width: usize) -> String {
        let size = (width as f64 * percent) as usize;
        let fill = Self::BAR_FILL.repeat(size);
        let none = Self::BAR_NONE.repeat(width - size);

        format!("`{fill}{none}`")
    }

    async fn __overview(&self, http: &Http, form: Form) -> Result<CreateEmbed> {
        let user = http.get_user(form.user).await?;
        let total = form.replies.len();

        let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
        let footer = CreateEmbedFooter::new(format!("Page 1 / {}", self.pages()));
        let mut description = format!("**Total Votes:** {total}\n\n");

        for (index, users) in &self.votes {
            let Some(Input::Choice(data)) = form.inputs.get(*index) else {
				continue;
			};

            let percent = Self::__percent(users.len(), total);
            let graph = Self::__graph(percent, 10);
            let votes = format!("{} votes ({:.1}%)", users.len(), percent * 100.0);

            description.push_str(&format!("{graph} **{}** - {votes}\n", data.label));
        }

        let embed = CreateEmbed::new()
            .author(author)
            .color(user.accent_colour.unwrap_or(BOT_COLOR))
            .description(description)
            .footer(footer)
            .thumbnail(user.face())
            .title("Poll Results: Overview");

        if let Ok(anchor) = form.anchor() {
            Ok(embed.url(anchor.to_string()))
        } else {
            Ok(embed)
        }
    }
    async fn __specific(&self, http: &Http, form: Form, page: usize) -> Result<CreateEmbed> {
        let index = page.saturating_sub(2);

        let Some((index, users)) = self.votes.get(index) else {
			return Err(Error::MissingValue(Value::Other("Entry")));
		};
        let Some(Input::Choice(data)) = form.inputs.get(*index) else {
			return Err(Error::MissingValue(Value::Other("Input")));
		};

        let user = http.get_user(form.user).await?;
        let total = form.replies.len();

        let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
        let footer = CreateEmbedFooter::new(format!("Page {page} / {}", self.pages()));

        let percent = Self::__percent(users.len(), total);
        let graph = Self::__graph(percent, 32);
        let votes = format!("{} votes ({:.1}%)", users.len(), percent * 100.0);
        let users = if form.content.hide_members {
            "*Users are hidden*".to_string()
        } else if users.is_empty() {
            "*No responses*".to_string()
        } else {
            users
                .iter()
                .fold(String::new(), |s, i| format!("{s}<@{i}>\n"))
        };
        let description = format!("{votes}\n{graph}\n\n**Users:**\n>>> {users}");

        let embed = CreateEmbed::new()
            .author(author)
            .color(user.accent_colour.unwrap_or(BOT_COLOR))
            .description(description)
            .footer(footer)
            .thumbnail(user.face())
            .title(format!("Poll Results: {}", data.label));

        if let Ok(anchor) = form.anchor() {
            Ok(embed.url(anchor.to_string()))
        } else {
            Ok(embed)
        }
    }
}

#[async_trait]
impl AsEmbedAsync<(Form, usize)> for ChoiceOutputData {
    async fn as_embed(&self, http: &Http, (form, page): (Form, usize)) -> Result<CreateEmbed> {
        if page == 1 {
            self.__overview(http, form).await
        } else {
            self.__specific(http, form, page).await
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResponseOutputData {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub answers: BTreeMap<UserId, Vec<String>>,
}

impl ResponseOutputData {
    pub fn new(form: &Form) -> Self {
        let mut answers = BTreeMap::new();

        for (user, reply) in &form.replies {
            let Reply::Response(data) = reply else {
				continue;
			};

            answers.insert(*user, data.clone());
        }

        Self { answers }
    }

    pub fn pages(&self) -> usize {
        self.answers.len() + 1
    }

    async fn __overview(&self, http: &Http, form: Form) -> Result<CreateEmbed> {
        let user = http.get_user(form.user).await?;

        let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
        let footer = CreateEmbedFooter::new(format!("Page 1 / {}", self.pages()));
        let description = format!("**Total Responses:** {}", self.answers.len());

        let embed = CreateEmbed::new()
            .author(author)
            .color(user.accent_colour.unwrap_or(BOT_COLOR))
            .description(description)
            .footer(footer)
            .thumbnail(user.face())
            .title("Poll Results: Overview");

        if let Ok(anchor) = form.anchor() {
            Ok(embed.url(anchor.to_string()))
        } else {
            Ok(embed)
        }
    }
    async fn __specific(&self, http: &Http, form: Form, page: usize) -> Result<CreateEmbed> {
        let index = page.saturating_sub(2);

        let Some((user, answers)) = self.answers.iter().nth(index) else {
			return Err(Error::MissingValue(Value::Other("Entry")))
		};

        let user = http.get_user(*user).await?;

        let author = if form.content.hide_members {
            CreateEmbedAuthor::new("Anonymous User").icon_url(user.default_avatar_url())
        } else {
            CreateEmbedAuthor::new(user.tag()).icon_url(user.face())
        };
        let color = if form.content.hide_members {
            BOT_COLOR
        } else {
            user.accent_colour.unwrap_or(BOT_COLOR)
        };
        let thumbnail = if form.content.hide_members {
            user.default_avatar_url()
        } else {
            user.face()
        };
        let title = if form.content.hide_members {
            format!("Poll Results: User #{}", page.saturating_sub(1))
        } else {
            format!("Poll Results: {}", user.name)
        };

        let footer = CreateEmbedFooter::new(format!("Page {page} / {}", self.pages()));
        let mut embed = CreateEmbed::new()
            .author(author)
            .color(color)
            .footer(footer)
            .thumbnail(thumbnail)
            .title(title);

        for (index, answer) in answers.iter().enumerate() {
            let Some(Input::Response(data)) = form.inputs.get(index) else {
				continue;
			};
            let answer = answer.replace(['\n', '\r', '\t'], " ");

            embed = embed.field(&data.label, format!("> {answer}"), false);
        }

        if let Ok(anchor) = form.anchor() {
            Ok(embed.url(anchor.to_string()))
        } else {
            Ok(embed)
        }
    }
}

#[async_trait]
impl AsEmbedAsync<(Form, usize)> for ResponseOutputData {
    async fn as_embed(&self, http: &Http, (form, page): (Form, usize)) -> Result<CreateEmbed> {
        if page == 1 {
            self.__overview(http, form).await
        } else {
            self.__specific(http, form, page).await
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RaffleOutputData {
    pub winner: Option<UserId>,
}

impl RaffleOutputData {
    pub fn new(form: &Form) -> Self {
        let users = form.replies.keys().copied().collect::<Vec<_>>();
        let winner = (!users.is_empty())
            .then(|| {
                let index = thread_rng().gen_range(0..users.len());
                users.get(index).copied()
            })
            .flatten();

        Self { winner }
    }

    #[allow(clippy::unused_self)]
    pub const fn pages(&self) -> usize {
        1
    }
}

#[async_trait]
impl AsEmbedAsync<Form> for RaffleOutputData {
    async fn as_embed(&self, http: &Http, form: Form) -> Result<CreateEmbed> {
        let user = http.get_user(form.user).await?;

        let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
        let footer = CreateEmbedFooter::new("Page 1 / 1");

        let total = format!("**Total Entries:** {}", form.replies.len());
        let winner = self.winner.map_or_else(
            || "**Winner:** *No responses*".to_string(),
            |winner| format!("**Winner:** <@{winner}>"),
        );
        let users = if form.content.hide_members {
            "*Users are hidden*".to_string()
        } else if form.replies.is_empty() {
            "*No responses*".to_string()
        } else {
            form.replies
                .iter()
                .map(|r| r.0)
                .fold(String::new(), |s, i| format!("{s}<@{i}>\n"))
        };
        let description = format!("{total}\n{winner}\n\n**Users:**\n>>> {users}");

        let embed = CreateEmbed::new()
            .author(author)
            .color(user.accent_colour.unwrap_or(BOT_COLOR))
            .description(description)
            .footer(footer)
            .thumbnail(user.face())
            .title("Poll Results: Overview");

        if let Ok(anchor) = form.anchor() {
            Ok(embed.url(anchor.to_string()))
        } else {
            Ok(embed)
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Output {
    Choice(ChoiceOutputData),
    Response(ResponseOutputData),
    Raffle(RaffleOutputData),
}

impl Output {
    pub fn new(form: &Form) -> Self {
        match form.kind {
            Kind::Choice => Self::Choice(ChoiceOutputData::new(form)),
            Kind::Response => Self::Response(ResponseOutputData::new(form)),
            Kind::Raffle => Self::Raffle(RaffleOutputData::new(form)),
        }
    }

    pub fn pages(&self) -> usize {
        match self {
            Self::Choice(data) => data.pages(),
            Self::Response(data) => data.pages(),
            Self::Raffle(data) => data.pages(),
        }
    }
    pub fn wrap_page(&self, page: usize) -> usize {
        let pages = self.pages();

        if page == 0 {
            pages
        } else if page > pages {
            1
        } else {
            page
        }
    }
}

impl AsButtonVec<(UserId, MessageId, usize)> for Output {
    fn as_buttons(
        &self,
        disabled: bool,
        (user, message, page): (UserId, MessageId, usize),
    ) -> Vec<CreateButton> {
        let page = self.wrap_page(page);
        let disabled = disabled || self.pages() == 1;

        let last = CreateButton::new(CustomId::new(BUTTON_LAST).arg(user).arg(message).arg(page))
            .disabled(disabled)
            .emoji('⬅')
            .style(ButtonStyle::Secondary);
        let next = CreateButton::new(CustomId::new(BUTTON_NEXT).arg(user).arg(message).arg(page))
            .disabled(disabled)
            .emoji('➡')
            .style(ButtonStyle::Secondary);

        vec![last, next]
    }
}

#[async_trait]
impl AsEmbedAsync<(Form, usize)> for Output {
    async fn as_embed(&self, http: &Http, (form, page): (Form, usize)) -> Result<CreateEmbed> {
        let page = self.wrap_page(page);

        match self {
            Self::Choice(data) => data.as_embed(http, (form, page)).await,
            Self::Response(data) => data.as_embed(http, (form, page)).await,
            Self::Raffle(data) => data.as_embed(http, form).await,
        }
    }
}
