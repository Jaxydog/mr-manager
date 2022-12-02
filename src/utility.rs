use serenity::prelude::GatewayIntents;

use crate::prelude::*;

pub mod anchor;
pub mod custom_id;
pub mod formatting;
pub mod handler;
pub mod logger;
pub mod req;
pub mod traits;

#[cfg(debug_assertions)]
pub const IS_DEV: bool = true;
#[cfg(not(debug_assertions))]
pub const IS_DEV: bool = false;

pub const BOT_COLOR: Color = Color::from_rgb(172, 90, 110);
pub const BOT_INTENTS: GatewayIntents = GatewayIntents::DIRECT_MESSAGES
    .union(GatewayIntents::GUILD_EMOJIS_AND_STICKERS)
    .union(GatewayIntents::GUILD_MEMBERS)
    .union(GatewayIntents::GUILD_MESSAGE_REACTIONS)
    .union(GatewayIntents::GUILD_MESSAGES)
    .union(GatewayIntents::GUILD_SCHEDULED_EVENTS)
    .union(GatewayIntents::GUILDS);

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Value {
    Anchor,
    Channel,
    Command,
    Component,
    CustomId,
    Data,
    Guild,
    Interaction,
    Member,
    Message,
    Modal,
    Role,
    User,
    Other(&'static str),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Other(s) => write!(f, "{s}"),
            k => write!(f, "{k:?}"),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    ReadWrite(std::io::Error),
    Decode(rmp_serde::decode::Error),
    Encode(rmp_serde::encode::Error),
    Serenity(serenity::Error),

    InvalidId(Value, String),
    InvalidValue(Value, String),
    MissingId(Value),
    MissingValue(Value),

    Other(&'static str),
}

macro_rules! from {
    ($err:path => $kind:path) => {
        impl From<$err> for Error {
            fn from(e: $err) -> Self {
                $kind(e)
            }
        }
    };
}

from!(std::io::Error => Self::ReadWrite);
from!(rmp_serde::decode::Error => Self::Decode);
from!(rmp_serde::encode::Error => Self::Encode);
from!(serenity::Error => Self::Serenity);

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::ReadWrite(e) => return e.fmt(f),
            Self::Decode(e) => return e.fmt(f),
            Self::Encode(e) => return e.fmt(f),
            Self::Serenity(e) => return e.fmt(f),

            Self::MissingId(k) => format!("Missing identifier: {k}<?>"),
            Self::InvalidId(k, s) => format!("Invalid identifier: {k}<{s}>"),
            Self::MissingValue(k) => format!("Missing value: {k}<?>"),
            Self::InvalidValue(k, s) => format!("Invalid value: {k}<{s}>"),

            Self::Other(s) => (*s).to_string(),
        };

        write!(f, "{text}")
    }
}

pub fn flag(name: &str) -> bool {
    let flag = format!("--{name}");

    std::env::args().any(|f| f == flag)
}
pub fn token() -> Result<String> {
    let key = if IS_DEV { "DEV_TOKEN" } else { "TOKEN" };

    std::env::var(key).map_err(|_| Error::MissingValue(Value::Other("Token")))
}
pub fn dev_guild() -> Result<GuildId> {
    let Ok(raw) = std::env::var("DEV_GUILD") else {
		return Err(Error::MissingId(Value::Guild));
	};
    let Ok(id) = raw.parse() else {
		return Err(Error::InvalidId(Value::Guild, raw));
	};

    Ok(GuildId::new(id))
}
