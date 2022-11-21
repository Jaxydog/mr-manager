use std::fmt::{self, Display, Formatter};

use chrono::Utc;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use serenity::{
    builder::{
        CreateActionRow, CreateAttachment, CreateButton, CreateEmbed, CreateEmbedAuthor,
        CreateInputText, CreateMessage, CreateModal, EditMessage,
    },
    model::prelude::{
        component::{ButtonStyle, InputTextStyle},
        ChannelId, GuildId, MessageId, ReactionType, UserId,
    },
    prelude::{CacheHttp, Context},
};

use crate::{
    utility::{
        storage::{insert_temp, remove_temp, Request, Storage},
        to_unix_str, Error, Result,
    },
    DEFAULT_COLOR,
};

pub const NAME: &str = "poll";

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Kind {
    Straw,
    Choice,
    Open,
}

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let raw = match self {
            Self::Straw => "Straw-Picking",
            Self::Choice => "Multiple Choice",
            Self::Open => "Open-Ended",
        };

        write!(f, "{raw}")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Poll {
    user_id: UserId,
    kind: Kind,
    inputs: Vec<Input>,
    outputs: Vec<Output>,
    content: Content,
    anchor: Option<Anchor>,
}

impl Poll {
    fn data_request_for(user_id: UserId) -> Request {
        Request::new(NAME, &user_id.to_string())
    }
    fn data_request(&self) -> Request {
        Self::data_request_for(self.user_id)
    }
    fn log_request_for(user_id: UserId, message_id: MessageId) -> Request {
        Request::new(&format!("{NAME}/{user_id}"), &message_id.to_string())
    }
    fn log_request(&self) -> Option<Request> {
        self.anchor
            .as_ref()
            .map(|anchor| Self::log_request_for(self.user_id, anchor.message_id))
    }
    fn temp_request_for(user_id: UserId) -> Request {
        Request::new(&format!("temp/{NAME}"), &user_id.to_string())
    }
    fn temp_request(&self) -> Request {
        Self::temp_request_for(self.user_id)
    }

    const fn is_sent(&self) -> bool {
        self.anchor.is_some()
    }
    const fn is_unsent(&self) -> bool {
        self.anchor.is_none()
    }

    const fn has_public_users(&self) -> bool {
        !self.content.hide_users
    }
    const fn has_private_users(&self) -> bool {
        self.content.hide_users
    }
    const fn has_public_results(&self) -> bool {
        !self.content.hide_results
    }
    const fn has_private_results(&self) -> bool {
        self.content.hide_results
    }

    async fn get(db: &mut Storage, user_id: UserId, kind: Kind) -> Result<Self> {
        let poll: Self = db.get(&Self::data_request_for(user_id)).await?;

        if poll.kind == kind {
            Ok(poll)
        } else {
            Err(Error::Other("Invalid poll type".into()))
        }
    }
    async fn get_sent(db: &mut Storage, user_id: UserId, kind: Kind) -> Result<Self> {
        let poll = Self::get(db, user_id, kind).await?;

        if poll.is_sent() {
            Ok(poll)
        } else {
            Err(Error::Other("Poll has not been sent".into()))
        }
    }
    async fn get_unsent(db: &mut Storage, user_id: UserId, kind: Kind) -> Result<Self> {
        let poll = Self::get(db, user_id, kind).await?;

        if poll.is_unsent() {
            Ok(poll)
        } else {
            Err(Error::Other("Poll has already been sent".into()))
        }
    }
    async fn save(&self, db: &mut Storage) -> Result<()> {
        db.insert(&self.data_request(), self).await
    }
    async fn archive(&self, db: &mut Storage) -> Result<()> {
        if let Some(request) = self.log_request() {
            db.insert(&request, self).await?;
            db.remove(&self.data_request()).await
        } else {
            Err(Error::Other("Poll has not been sent".into()))
        }
    }
    async fn close(&self, db: &mut Storage, ctx: &Context) -> Result<()> {
        let Some(anchor) = self.anchor.as_ref() else {
			return Err(Error::Other("Poll has not been sent".into()))
		};

        let guild = anchor.guild_id.to_partial_guild(ctx.http()).await?;
        let channels = guild.channels(ctx.http()).await?;
        let Some(channel) = channels.get(&anchor.channel_id).cloned() else {
			return Err(Error::MissingChannel)
		};
        let mut message = channel.message(ctx.http(), anchor.message_id).await?;
        let results = self.create_results_string(ctx).await?;
        let request = self.temp_request();

        insert_temp(&request, &results).await?;
        let file = CreateAttachment::path(request.full_path()).await?;

        let mut edit = EditMessage::new().components(vec![]);

        for button in self.create_buttons(true) {
            edit = edit.button(button);
        }

        message.edit(ctx.http(), edit).await?;

        if self.has_public_results() {
            channel
                .send_files(
                    ctx.http(),
                    [file],
                    CreateMessage::new().reference_message(&message),
                )
                .await?;
        } else if let Ok(channel) = self.user_id.create_dm_channel(ctx.http()).await {
            channel
                .send_files(ctx.http(), [file], CreateMessage::new())
                .await?;
        } else {
            channel
                .send_files(
                    ctx.http(),
                    [file],
                    CreateMessage::new()
                        .reference_message(&message)
                        .content("Poll author has DMs disabled"),
                )
                .await?;
        }

        remove_temp(&request).await?;
        self.archive(db).await
    }

    async fn create_embed(&self, ctx: &Context) -> Result<CreateEmbed> {
        let user = self.user_id.to_user(ctx.http()).await?;
        let ms = i64::from(self.content.hours) * 60 * 60 * 1000;

        let closes = format!("**Closes:** {}", to_unix_str(ms, "R"));
        let anonymous = format!("**Anonymous:** {}", self.has_private_users());
        let results = format!("**Hidden Results:** {}", self.has_private_results());
        let content = self.content.description.replace(['\t', '\n', '\r'], " ");

        let description = format!("{closes}\n{anonymous}\n{results}\n\n> {content}");

        Ok(CreateEmbed::new()
            .author(CreateEmbedAuthor::new(user.tag()).icon_url(user.face()))
            .color(user.accent_colour.unwrap_or(DEFAULT_COLOR))
            .description(description)
            .thumbnail(user.face())
            .title(&self.content.title))
    }
    fn create_buttons(&self, disabled: bool) -> Vec<CreateButton> {
        match self.kind {
            Kind::Straw => vec![
                CreateButton::new(format!("{NAME}_straw;{}", self.user_id))
                    .disabled(disabled)
                    .emoji('✋')
                    .label("Participate")
                    .style(ButtonStyle::Primary),
                CreateButton::new(format!("{NAME}_straw_info"))
                    .disabled(disabled)
                    .emoji('ℹ')
                    .label(format!("About {} Polls", Kind::Straw))
                    .style(ButtonStyle::Secondary),
            ],
            Kind::Choice => {
                let mut buttons = vec![];

                for input in self.inputs.iter().take(25) {
                    let Input::Choice(id, label, icon) = input else {
						continue;
					};

                    let custom_id = format!("{NAME}_choice;{};{id}", self.user_id);
                    let button = CreateButton::new(custom_id)
                        .disabled(disabled)
                        .label(label)
                        .style(ButtonStyle::Secondary);

                    buttons.push(if let Some(emoji) = icon {
                        button.emoji(emoji.clone())
                    } else {
                        button
                    });
                }

                buttons
            }
            Kind::Open => vec![
                CreateButton::new(format!("{NAME}_open;{}", self.user_id))
                    .disabled(disabled)
                    .emoji('📨')
                    .label("Submit")
                    .style(ButtonStyle::Primary),
                CreateButton::new(format!("{NAME}_open_info"))
                    .disabled(disabled)
                    .emoji('ℹ')
                    .label(format!("About {} Polls", Kind::Open))
                    .style(ButtonStyle::Secondary),
            ],
        }
    }
    fn create_remove_buttons(&self) -> Vec<CreateButton> {
        let mut buttons = vec![];

        for input in self.inputs.iter().take(25) {
            let (id, label) = match input {
                Input::Choice(id, label, _) | Input::Open(id, label, _) => (*id, label),
            };

            let custom_id = format!("{NAME}_remove;{};{id}", self.user_id);

            buttons.push(
                CreateButton::new(custom_id)
                    .label(label)
                    .style(ButtonStyle::Danger),
            );
        }

        buttons
    }
    fn create_modal(&self) -> Option<CreateModal> {
        if self.kind != Kind::Open {
            return None;
        }

        let mut components = vec![];

        for input in self.inputs.iter().take(5) {
            let Input::Open(id, label, hint) = input else {
				continue;
			};

            let input = CreateInputText::new(InputTextStyle::Paragraph, label, id.to_string());

            components.push(CreateActionRow::InputText(
                if let Some(label) = hint.as_ref() {
                    input.placeholder(label)
                } else {
                    input
                },
            ));
        }

        let custom_id = format!("{NAME}_modal;{}", self.user_id);
        Some(CreateModal::new(custom_id, "Submit poll response").components(components))
    }

    fn create_results_header(&self) -> Result<String> {
        self.anchor.as_ref().map_or_else(
            || Err(Error::Other("Poll has not been sent".into())),
            |anchor| {
                let header = format!("Results for poll \"{}\"", self.content.title);
                let created = to_unix_str(anchor.message_id.created_at().timestamp_millis(), "f");
                let closed = to_unix_str(Utc::now().timestamp_millis(), "f");

                Ok(format!(
                    "{header}\nCreated: {created}\nClosed: {closed}\n\n"
                ))
            },
        )
    }
    async fn __create_results_straw(&self, ctx: &Context) -> Result<String> {
        let total = format!("Total participating: {}", self.outputs.len());

        if self.outputs.is_empty() {
            return Ok(format!("{total}\nNo winner"));
        }

        let list = if self.has_public_users() {
            let mut list = "Users: ".to_string();

            for output in &self.outputs {
                let Output::Straw(id) = output else {
					continue;
				};

                let user = id.to_user(ctx.http()).await?;
                list.push_str(&format!("{}, ", user.tag()));
            }

            list.trim_end_matches(", ").to_string()
        } else {
            "*Users hidden*".into()
        };

        let index = thread_rng().gen_range(0..self.outputs.len());
        let victor = self.outputs[index]
            .user_id()
            .to_user(ctx.http())
            .await?
            .tag();

        let winner = format!("Winner: {victor}");

        Ok(format!("{total}\n{list}\n{winner}"))
    }
    #[allow(clippy::cast_precision_loss)]
    async fn __create_results_choice(&self, ctx: &Context) -> Result<String> {
        let responses = format!("Total responses: {}", self.outputs.len());
        let mut results = vec![];

        for input in self.inputs.iter().take(25) {
            let Input::Choice(id, label, _) = input else {
				continue;
			};
            let valid = self.outputs.iter().filter(|o| match o {
                Output::Choice(_, i) => i == id,
                _ => false,
            });

            let count = valid.clone().count();

            let percent = if count == 0 {
                (count as f64 / self.outputs.len() as f64) * 100.0
            } else {
                0.0
            };

            let users = if self.has_public_users() {
                let mut users = vec![];

                for user_id in valid.map(Output::user_id) {
                    users.push(user_id.to_user(ctx.http()).await?.tag());
                }

                users.join(", ")
            } else {
                "*Users hidden*".into()
            };

            let text = format!("{label}\n\tUsers: {users}\n\tTotal: {count} ({percent:.2}%)");
            results.push((count, text));
        }

        results.sort_unstable_by_key(|(c, _)| *c);
        results.reverse();

        let results = results
            .into_iter()
            .map(|(_, t)| t)
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(format!("{responses}\n\n{results}"))
    }
    async fn __create_results_open(&self, ctx: &Context) -> Result<String> {
        let responses = format!("Total responses: {}", self.outputs.len());
        let mut results = vec![];

        for input in self.inputs.iter().take(25) {
            let Input::Open(input_id, label, _) = input else {
				continue;
			};

            let mut result = format!("{label}\n");

            for output in &self.outputs {
                let Output::Open(user_id, answers) = output else {
					continue;
				};

                for (id, answer) in answers {
                    if input_id != id {
                        continue;
                    }

                    if self.has_public_users() {
                        let tag = user_id.to_user(ctx.http()).await?.tag();
                        result.push_str(&format!("\t{tag} - {answer}\n"));
                    } else {
                        result.push_str(&format!("\t{answer}\n"));
                    }
                }
            }

            results.push(result.trim().to_string());
        }

        Ok(format!("{responses}\n\n{}", results.join("\n\n")))
    }
    async fn create_results_string(&self, ctx: &Context) -> Result<String> {
        if self.is_unsent() {
            return Err(Error::Other("Poll has not been sent".into()));
        }

        let mut string = self.create_results_header()?;

        string.push_str(&match self.kind {
            Kind::Straw => self.__create_results_straw(ctx).await?,
            Kind::Choice => self.__create_results_choice(ctx).await?,
            Kind::Open => self.__create_results_open(ctx).await?,
        });

        Ok(string)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Content {
    title: String,
    description: String,
    hours: u8,
    hide_users: bool,
    hide_results: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Anchor {
    guild_id: GuildId,
    channel_id: ChannelId,
    message_id: MessageId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Input {
    Choice(u8, String, Option<ReactionType>),
    Open(u8, String, Option<String>),
}

impl Input {
    pub const fn kind(&self) -> Kind {
        match self {
            Self::Choice(..) => Kind::Choice,
            Self::Open(..) => Kind::Open,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Output {
    Straw(UserId),
    Choice(UserId, u8),
    Open(UserId, Vec<(u8, String)>),
}

impl Output {
    pub const fn kind(&self) -> Kind {
        match self {
            Self::Straw(..) => Kind::Straw,
            Self::Choice(..) => Kind::Choice,
            Self::Open(..) => Kind::Open,
        }
    }
    pub const fn user_id(&self) -> UserId {
        match self {
            Output::Straw(id) | Output::Choice(id, _) | Output::Open(id, _) => *id,
        }
    }
}
