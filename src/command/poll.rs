use std::num::NonZeroU8;

use const_format::formatcp;
use serde::{Deserialize, Serialize};
use serenity::{
    builder::{
        CreateActionRow, CreateButton, CreateEmbed, CreateEmbedAuthor, CreateInputText,
        CreateMessage, CreateModal, EditMessage,
    },
    model::{
        prelude::{
            component::{ButtonStyle, InputTextStyle},
            ChannelId, GuildChannel, GuildId, Message, MessageId, PartialGuild, ReactionType,
            UserId,
        },
        Color,
    },
    prelude::{CacheHttp, Context},
};

use crate::{
    utility::{
        storage::{Request, Storage},
        to_unix_str, Error, Result,
    },
    DEFAULT_COLOR,
};

pub const NAME: &str = "poll";
pub const MULTIPLE_CHOICE_BUTTON: &str = formatcp!("{NAME}_choice");
pub const TEXT_RESPONSE_BUTTON: &str = formatcp!("{NAME}_text");
pub const TEXT_RESPONSE_INFO_BUTTON: &str = formatcp!("{TEXT_RESPONSE_BUTTON}_info");
pub const TEXT_RESPONSE_MODAL: &str = formatcp!("{NAME}_text");
pub const RANDOM_WINNER_BUTTON: &str = formatcp!("{NAME}_random");
pub const RANDOM_WINNER_INFO_BUTTON: &str = formatcp!("{RANDOM_WINNER_BUTTON}_info");
pub const REMOVE_BUTTON: &str = formatcp!("{NAME}_remove");
pub const RESULTS_BUTTON: &str = formatcp!("{NAME}_results");

#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Kind {
    MultipleChoice,
    TextResponse,
    RandomWinner,
}

impl Kind {
    const fn has_inputs(self) -> bool {
        self.max_inputs() > 0
    }
    const fn max_inputs(self) -> usize {
        match self {
            Self::MultipleChoice => 10,
            Self::TextResponse => 5,
            Self::RandomWinner => 3,
        }
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, character) in format!("{self:?}").char_indices() {
            if character.is_uppercase() && index > 0 {
                write!(f, " ")?;
            }

            write!(f, "{character}")?;
        }

        Ok(())
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Input {
    MultipleChoice {
        input_id: u8,
        label: String,
        icon: Option<ReactionType>,
    },
    TextResponse {
        input_id: u8,
        label: String,
        placeholder: Option<String>,
    },
    RandomWinner {
        input_id: u8,
        label: String,
        icon: Option<ReactionType>,
    },
}

impl Input {
    const fn kind(&self) -> Kind {
        match self {
            Self::MultipleChoice { .. } => Kind::MultipleChoice,
            Self::TextResponse { .. } => Kind::TextResponse,
            Self::RandomWinner { .. } => Kind::RandomWinner,
        }
    }
    const fn input_id(&self) -> u8 {
        match self {
            Self::MultipleChoice { input_id, .. }
            | Self::TextResponse { input_id, .. }
            | Self::RandomWinner { input_id, .. } => *input_id,
        }
    }
    const fn label(&self) -> &String {
        match self {
            Self::MultipleChoice { label, .. }
            | Self::TextResponse { label, .. }
            | Self::RandomWinner { label, .. } => label,
        }
    }
}

#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
enum Reply {
    MultipleChoice {
        user_id: UserId,
        input_id: u8,
    },
    TextResponse {
        user_id: UserId,
        response: Vec<(u8, String)>,
    },
    RandomWinner {
        user_id: UserId,
        input_id: u8,
    },
}

impl Reply {
    const fn kind(&self) -> Kind {
        match self {
            Self::MultipleChoice { .. } => Kind::MultipleChoice,
            Self::TextResponse { .. } => Kind::TextResponse,
            Self::RandomWinner { .. } => Kind::RandomWinner,
        }
    }
    const fn user_id(&self) -> UserId {
        match self {
            Self::MultipleChoice { user_id, .. }
            | Self::TextResponse { user_id, .. }
            | Self::RandomWinner { user_id, .. } => *user_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Content {
    kind: Kind,
    title: String,
    description: String,
    hours: NonZeroU8,
    hide_users: bool,
    hide_results: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Anchor {
    guild_id: GuildId,
    channel_id: ChannelId,
    message_id: MessageId,
}

impl Anchor {
    fn as_link(&self) -> String {
        format!(
            "https://discord.com/channels/{}/{}/{}/",
            self.guild_id, self.channel_id, self.message_id
        )
    }

    async fn resolve_guild(&self, ctx: &Context) -> Result<PartialGuild> {
        Ok(self.guild_id.to_partial_guild(ctx.http()).await?)
    }
    async fn resolve_channel(&self, ctx: &Context) -> Result<GuildChannel> {
        let guild = self.resolve_guild(ctx).await?;
        let mut channels = guild.channels(ctx.http()).await?;

        channels
            .remove(&self.channel_id)
            .ok_or(Error::MissingChannel)
    }
    async fn resolve_message(&self, ctx: &Context) -> Result<Message> {
        let channel = self.resolve_channel(ctx).await?;
        Ok(channel.message(ctx.http(), self.message_id).await?)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Poll {
    user_id: UserId,
    inputs: Vec<Input>,
    replies: Vec<Reply>,
    content: Content,
    anchor: Option<Anchor>,
}

impl Poll {
    const ERROR_ANCHORED: Error = Error::Other("Poll has been sent");
    const ERROR_UNANCHORED: Error = Error::Other("Poll has not been sent");
    const ERROR_ARCHIVED: Error = Error::Other("Poll has been archived");
    const ERROR_UNARCHIVED: Error = Error::Other("Poll has not been archived");
    const ERROR_WRONG_KIND: Error = Error::Other("Unexpected poll type");

    const fn anchor(&self) -> Option<&Anchor> {
        self.anchor.as_ref()
    }

    fn is_of(&self, kind: Kind) -> bool {
        self.content.kind == kind
    }
    fn ensure_is_of(&self, kind: Kind) -> Result<()> {
        self.is_of(kind).then_some(()).ok_or(Self::ERROR_WRONG_KIND)
    }

    const fn is_anchored(&self) -> bool {
        self.anchor.is_some()
    }
    const fn is_unanchored(&self) -> bool {
        self.anchor.is_none()
    }

    async fn is_archived(&self, db: &Storage) -> bool {
        let Ok(request) = self.new_archive_request() else {
			return false;
		};

        db.contains(&request).await
    }
    async fn is_unarchived(&self, db: &Storage) -> bool {
        db.contains(&self.new_data_request()).await
    }

    fn new_archive_request_for(user_id: UserId, message_id: MessageId) -> Request {
        Request::new(&format!("{NAME}\\{user_id}"), &message_id.to_string())
    }
    fn new_data_request_for(user_id: UserId) -> Request {
        Request::new(NAME, &user_id.to_string())
    }
    fn new_temp_request_for(user_id: UserId) -> Request {
        Request::new(&format!("temp\\{NAME}"), &user_id.to_string())
    }

    fn new_archive_request(&self) -> Result<Request> {
        self.anchor()
            .map(|a| Self::new_archive_request_for(self.user_id, a.message_id))
            .ok_or(Self::ERROR_UNANCHORED)
    }
    fn new_data_request(&self) -> Request {
        Self::new_data_request_for(self.user_id)
    }
    fn new_temp_request(&self) -> Request {
        Self::new_temp_request_for(self.user_id)
    }

    async fn get(db: &mut Storage, user_id: UserId) -> Result<Self> {
        db.get(&Self::new_data_request_for(user_id)).await
    }
    async fn get_anchored(db: &mut Storage, user_id: UserId) -> Result<Self> {
        let poll = Self::get(db, user_id).await?;

        poll.is_anchored()
            .then_some(poll)
            .ok_or(Self::ERROR_UNANCHORED)
    }
    async fn get_unanchored(db: &mut Storage, user_id: UserId) -> Result<Self> {
        let poll = Self::get(db, user_id).await?;

        poll.is_unanchored()
            .then_some(poll)
            .ok_or(Self::ERROR_ANCHORED)
    }
    async fn get_archived(
        db: &mut Storage,
        user_id: UserId,
        message_id: MessageId,
    ) -> Result<Self> {
        db.get(&Self::new_archive_request_for(user_id, message_id))
            .await
    }

    async fn save(&self, db: &mut Storage) -> Result<()> {
        db.insert(&self.new_data_request(), self).await
    }
    async fn archive(&self, db: &mut Storage) -> Result<()> {
        let request = self.new_archive_request()?;
        db.insert(&request, self).await?;
        db.remove(&self.new_data_request()).await
    }
    async fn unarchive(&self, db: &mut Storage) -> Result<()> {
        let request = self.new_archive_request()?;
        db.insert(&self.new_data_request(), self).await?;
        db.remove(&request).await
    }

    fn __build_display_buttons_multiple_choice(&self, enabled: bool) -> Vec<CreateButton> {
        let count = self.content.kind.max_inputs();
        let mut buttons = vec![];

        for input in self.inputs.iter().take(count) {
            let Input::MultipleChoice { input_id, label, icon } = input else {
				continue;
			};

            let custom_id = format!("{MULTIPLE_CHOICE_BUTTON};{};{input_id}", self.user_id);
            let button = CreateButton::new(custom_id)
                .disabled(!enabled)
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
    fn __build_display_buttons_text_response(&self, enabled: bool) -> Vec<CreateButton> {
        vec![
            CreateButton::new(format!("{TEXT_RESPONSE_BUTTON};{}", self.user_id))
                .disabled(!enabled)
                .emoji('📨')
                .label("Submit")
                .style(ButtonStyle::Primary),
            CreateButton::new(TEXT_RESPONSE_INFO_BUTTON)
                .disabled(!enabled)
                .emoji('ℹ')
                .label(format!("About {} Polls", Kind::TextResponse))
                .style(ButtonStyle::Secondary),
        ]
    }
    fn __build_display_buttons_random_winner(&self, enabled: bool) -> Vec<CreateButton> {
        let count = self.content.kind.max_inputs();
        let mut buttons = vec![];

        for input in self.inputs.iter().take(count) {
            let Input::RandomWinner { input_id, label, icon } = input else {
				continue;
			};

            let custom_id = format!("{RANDOM_WINNER_BUTTON};{};{input_id}", self.user_id);
            let button = CreateButton::new(custom_id)
                .disabled(!enabled)
                .label(label)
                .style(ButtonStyle::Secondary);

            buttons.push(if let Some(emoji) = icon {
                button.emoji(emoji.clone())
            } else {
                button
            });
        }

        buttons.push(
            CreateButton::new(RANDOM_WINNER_INFO_BUTTON)
                .disabled(!enabled)
                .emoji('ℹ')
                .label(format!("About {} Polls", Kind::RandomWinner))
                .style(ButtonStyle::Secondary),
        );

        buttons
    }
    fn __build_display_buttons(&self, enabled: bool) -> Vec<CreateButton> {
        match self.content.kind {
            Kind::MultipleChoice => self.__build_display_buttons_multiple_choice(enabled),
            Kind::TextResponse => self.__build_display_buttons_text_response(enabled),
            Kind::RandomWinner => self.__build_display_buttons_random_winner(enabled),
        }
    }
    async fn build_display(&self, ctx: &Context, enabled: bool) -> Result<CreateMessage> {
        let user = self.user_id.to_user(ctx.http()).await?;
        let millis = i64::from(u8::from(self.content.hours)) * 60 * 60 * 1000;
        let author = CreateEmbedAuthor::new(user.tag()).icon_url(user.face());
        let color = user.accent_colour.unwrap_or(DEFAULT_COLOR);
        let mut description = String::new();

        {
            use std::fmt::Write;
            let f = &mut description;

            writeln!(f, "**Closes:** {}", to_unix_str(millis, "R"))?;
            writeln!(f, "**Users hidden:** {}", self.content.hide_users)?;
            writeln!(f, "**Results hidden:** {}", self.content.hide_results)?;
            writeln!(f, "\n> {}", self.content.description)?;
        }

        let embed = CreateEmbed::new()
            .author(author)
            .color(color)
            .description(description)
            .thumbnail(user.face())
            .title(&self.content.title);

        let mut message = CreateMessage::new().embed(embed);

        for button in self.__build_display_buttons(enabled) {
            message = message.button(button);
        }

        Ok(message)
    }
    fn build_modal(&self) -> Result<CreateModal> {
        self.ensure_is_of(Kind::TextResponse)?;

        let count = self.content.kind.max_inputs();
        let mut components = vec![];

        for input in self.inputs.iter().take(count) {
            let Input::TextResponse { input_id, label, placeholder } = input else {
				continue;
			};

            let custom_id = input_id.to_string();
            let input = CreateInputText::new(InputTextStyle::Paragraph, label, custom_id);

            components.push(CreateActionRow::InputText(
                if let Some(label) = placeholder {
                    input.placeholder(label)
                } else {
                    input
                },
            ));
        }

        let custom_id = format!("{TEXT_RESPONSE_MODAL};{}", self.user_id);
        Ok(CreateModal::new(custom_id, "Submit poll response").components(components))
    }
    fn build_remove(&self) -> CreateMessage {
        let inputs = self.content.kind.max_inputs();
        let embed = CreateEmbed::new().color(Color::RED).title("Remove inputs!");
        let mut message = CreateMessage::new().embed(embed);

        for input in self.inputs.iter().take(inputs) {
            let input_id = input.input_id();
            let label = input.label();
            let custom_id = format!("{REMOVE_BUTTON};{};{input_id}", self.user_id);

            message = message.button(
                CreateButton::new(custom_id)
                    .label(label)
                    .style(ButtonStyle::Danger),
            );
        }

        message
    }
    async fn build_results(&self, ctx: &Context) -> Result<CreateMessage> {
        let user = self.user_id.to_user(ctx.http()).await?;
        let color = user.accent_colour.unwrap_or(DEFAULT_COLOR);
        let mut embed = CreateEmbed::new().color(color).title("Poll is now closed!");

        if self.content.hide_results {
            embed = embed.description("*Results may only be viewed by the poll author.*");
        }

        let button = CreateButton::new(format!("{RESULTS_BUTTON};{}", self.user_id))
            .emoji('📊')
            .label("View Results")
            .style(ButtonStyle::Primary);

        Ok(CreateMessage::new().embed(embed).button(button))
    }

    async fn close(&self, db: &mut Storage, ctx: &Context) -> Result<()> {
        let anchor = self.anchor().ok_or(Self::ERROR_UNANCHORED)?;
        let results = self.build_results(ctx).await?;
        let mut message = anchor.resolve_message(ctx).await?;
        let mut edit = EditMessage::new().components(vec![]);

        for button in self.__build_display_buttons(false) {
            edit = edit.button(button);
        }

        message.edit(ctx.http(), edit).await?;
        anchor.channel_id.send_message(ctx.http(), results).await?;
        self.archive(db).await
    }
}
