use std::fmt::Display;

use chrono::Utc;

pub mod logger;
pub mod storage;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    RustMPDecode(rmp_serde::decode::Error),
    RustMPEncode(rmp_serde::encode::Error),
    Serenity(serenity::Error),
    Io(tokio::io::Error),
    InvalidCommandData,
    InvalidDevGuild,
    InvalidRequest,
    MissingCommand,
    MissingCommandData,
    MissingInteraction,
    MissingDevGuild,
    Other(String),
}

impl From<rmp_serde::decode::Error> for Error {
    fn from(error: rmp_serde::decode::Error) -> Self {
        Self::RustMPDecode(error)
    }
}

impl From<rmp_serde::encode::Error> for Error {
    fn from(error: rmp_serde::encode::Error) -> Self {
        Self::RustMPEncode(error)
    }
}

impl From<serenity::Error> for Error {
    fn from(error: serenity::Error) -> Self {
        Self::Serenity(error)
    }
}

impl From<tokio::io::Error> for Error {
    fn from(error: tokio::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<String> for Error {
    fn from(string: String) -> Self {
        Self::Other(string)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::RustMPDecode(e) => return e.fmt(f),
            Self::RustMPEncode(e) => return e.fmt(f),
            Self::Serenity(e) => return e.fmt(f),
            Self::Io(e) => return e.fmt(f),
            Self::InvalidCommandData => "The received command is contains invalid data",
            Self::InvalidDevGuild => "Invalid development guild identifier",
            Self::InvalidRequest => "Invalid request configuration",
            Self::MissingCommand => "The received command is not registered",
            Self::MissingCommandData => "The received command is missing data",
            Self::MissingInteraction => "The received interaction is not handled",
            Self::MissingDevGuild => "Missing development guild identifier",
            Self::Other(s) => s,
        };

        write!(f, "{message}")
    }
}

pub fn to_unix_str(millis: i64, flag: &str) -> String {
    let time = Utc::now().timestamp_millis() + millis;
    let trimmed = time / 1000;
    format!("<t:{trimmed}:{flag}>")
}
