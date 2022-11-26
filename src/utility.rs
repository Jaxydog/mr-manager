use std::collections::VecDeque;

use crate::prelude::*;

pub mod handler;
pub mod logging;
pub mod request;

pub const BOT_COLOR: Color = Color::from_rgb(172, 90, 110);
pub const INTENTS: GatewayIntents = GatewayIntents::DIRECT_MESSAGES
    .union(GatewayIntents::GUILD_EMOJIS_AND_STICKERS)
    .union(GatewayIntents::GUILD_MEMBERS)
    .union(GatewayIntents::GUILD_MESSAGE_REACTIONS)
    .union(GatewayIntents::GUILD_MESSAGES)
    .union(GatewayIntents::GUILD_SCHEDULED_EVENTS)
    .union(GatewayIntents::GUILDS);

#[must_use]
pub fn flag_present(name: &str) -> bool {
    let flag = format!("--{name}");

    std::env::args().any(|arg| arg == flag)
}
#[must_use]
pub const fn is_dev() -> bool {
    cfg!(debug_assertions)
}

pub fn token() -> Result<String> {
    let key = if is_dev() { "DEV_TOKEN" } else { "TOKEN" };

    std::env::var(key).map_err(|_| Error::Other("Missing client token"))
}
pub fn dev_guild() -> Result<GuildId> {
    let Ok(raw) = std::env::var("DEV_GUILD") else {
		return Err(Error::Other("Missing development guild"))
	};
    let Ok(id) = raw.parse() else {
		return Err(Error::Other("Invalid development guild"))
	};

    Ok(GuildId::new(id))
}

pub fn parse_cid(cid: &str) -> Result<(String, String, Vec<String>)> {
    let mut parts: VecDeque<&str> = cid.split(';').collect();
    let name = parts
        .pop_front()
        .ok_or_else(|| Error::InvalidCustomId(cid.to_string()))?;
    let source = name
        .split('_')
        .next()
        .ok_or_else(|| Error::InvalidCustomId(cid.to_string()))?;

    Ok((
        name.to_string(),
        source.to_string(),
        parts.into_iter().map(ToString::to_string).collect(),
    ))
}

#[must_use]
pub fn timestamp_str(ms: i64, flag: &str) -> String {
    let time = Utc::now().timestamp_millis() + ms;
    let secs = time / 1000;

    format!("<t:{secs}:{flag}>")
}

pub type Result<T> = core::result::Result<T, Error>;

#[non_exhaustive]
#[derive(Debug)]
pub enum Error {
    Decode(rmp_serde::decode::Error),
    Encode(rmp_serde::encode::Error),
    Format(std::fmt::Error),
    InOut(std::io::Error),
    Serenity(serenity::Error),

    InvalidInteraction(InteractionId),

    InvalidCommand(String),
    InvalidCommandData(String, &'static str),
    MissingCommandData(String, &'static str),

    InvalidComponent(String),
    InvalidComponentData(String, &'static str),
    MissingComponentData(String, &'static str),

    InvalidModal(String),
    InvalidModalData(String, &'static str),
    MissingModalData(String, &'static str),

    InvalidCustomId(String),
    InvalidCustomIdData(String, &'static str),
    MissingCustomIdData(String, &'static str),

    InvalidRequest(String),

    InvalidRole(RoleId),
    InvalidGuild(GuildId),
    InvalidChannel(ChannelId),
    InvalidMessage(MessageId),
    InvalidMember(UserId),
    MissingRole(RoleId),
    MissingGuild(GuildId),
    MissingChannel(ChannelId),
    MissingMessage(MessageId),
    MissingMember(UserId),

    Other(&'static str),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Decode(e) => return e.fmt(f),
            Self::Encode(e) => return e.fmt(f),
            Self::Format(e) => return e.fmt(f),
            Self::InOut(e) => return e.fmt(f),
            Self::Serenity(e) => return e.fmt(f),

            Self::InvalidInteraction(id) => format!("Invalid interaction: {id}"),

            Self::InvalidCommand(name) => format!("Invalid command: {name}"),
            Self::InvalidCommandData(name, data) => {
                format!("Invalid command data: {name}::{data}")
            }
            Self::MissingCommandData(name, data) => {
                format!("Missing command data: {name}::{data}")
            }

            Self::InvalidComponent(cid) => format!("Invalid component: {cid}"),
            Self::InvalidComponentData(cid, data) => {
                format!("Invalid component data: {cid}::{data}")
            }
            Self::MissingComponentData(cid, data) => {
                format!("Missing component data: {cid}::{data}")
            }

            Self::InvalidModal(cid) => format!("Invalid modal: {cid}"),
            Self::InvalidModalData(cid, data) => {
                format!("Invalid modal data: {cid}::{data}")
            }
            Self::MissingModalData(cid, data) => {
                format!("Missing modal data: {cid}::{data}")
            }

            Self::InvalidCustomId(cid) => format!("Invalid custom identifier: {cid}"),
            Self::InvalidCustomIdData(cid, data) => {
                format!("Invalid custom identifier data: {cid}::{data}")
            }
            Self::MissingCustomIdData(cid, data) => {
                format!("Missing custom identifier data: {cid}::{data}")
            }

            Self::InvalidRequest(path) => format!("Invalid request: {path}"),

            Self::InvalidRole(id) => format!("Invalid role identifier: {id}"),
            Self::InvalidGuild(id) => format!("Invalid guild identifier: {id}"),
            Self::InvalidChannel(id) => format!("Invalid channel identifier: {id}"),
            Self::InvalidMessage(id) => format!("Invalid message identifier: {id}"),
            Self::InvalidMember(id) => format!("Invalid member identifier: {id}"),
            Self::MissingRole(id) => format!("Missing role identifier: {id}"),
            Self::MissingGuild(id) => format!("Missing guild identifier: {id}"),
            Self::MissingChannel(id) => format!("Missing channel identifier: {id}"),
            Self::MissingMessage(id) => format!("Missing message identifier: {id}"),
            Self::MissingMember(id) => format!("Missing member identifier: {id}"),

            Self::Other(s) => (*s).to_string(),
        };

        write!(f, "{text}")
    }
}

macro_rules! from_err {
    ($name:path, $from:path) => {
        impl From<$from> for Error {
            fn from(e: $from) -> Self {
                $name(e)
            }
        }
    };
}

from_err!(Error::Decode, rmp_serde::decode::Error);
from_err!(Error::Encode, rmp_serde::encode::Error);
from_err!(Error::Format, std::fmt::Error);
from_err!(Error::InOut, std::io::Error);
from_err!(Error::Serenity, serenity::Error);
