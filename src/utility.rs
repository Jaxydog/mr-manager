use std::fmt::Display;

pub mod logger;
pub mod storage;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    RustMPDecode(rmp_serde::decode::Error),
    RustMPEncode(rmp_serde::encode::Error),
    Serenity(serenity::Error),
    Io(tokio::io::Error),
    InvalidRequest,
    MissingDevGuild,
    InvalidDevGuild,
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

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::RustMPDecode(e) => return e.fmt(f),
            Self::RustMPEncode(e) => return e.fmt(f),
            Self::Serenity(e) => return e.fmt(f),
            Self::Io(e) => return e.fmt(f),
            Self::InvalidRequest => "Invalid request configuration",
            Self::MissingDevGuild => "Missing development guild identifier",
            Self::InvalidDevGuild => "Invalid development guild identifier",
        };

        write!(f, "{message}")
    }
}
