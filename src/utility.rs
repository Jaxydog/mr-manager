use std::fmt::Display;

pub mod logger;
pub mod storage;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Serenity(serenity::Error),
    MissingDevGuild,
    InvalidDevGuild,
}

impl From<serenity::Error> for Error {
    fn from(error: serenity::Error) -> Self {
        Self::Serenity(error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::Serenity(e) => return e.fmt(f),
            Self::MissingDevGuild => "Missing development guild identifier".to_string(),
            Self::InvalidDevGuild => "Invalid development guild identifier".to_string(),
        };

        write!(f, "{message}")
    }
}
