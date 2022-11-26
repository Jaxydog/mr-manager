use std::num::NonZeroU64;

use crate::prelude::*;

pub const NAME: &str = "poll";

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct InputId(u8);

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Kind {
    MultipleChoice,
    FreeResponse,
    RandomWinner,
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
#[derive(Debug, Serialize, Deserialize)]
pub enum Input {
    MultipleChoice {
        input_id: InputId,
        label: String,
        icon: ReactionType,
    },
    FreeResponse {
        input_id: InputId,
        label: String,
        placeholder: String,
    },
    RandomWinner {
        input_id: InputId,
        label: String,
        icon: ReactionType,
    },
}

impl Input {
    #[must_use]
    pub const fn kind(&self) -> Kind {
        match self {
            Self::MultipleChoice { .. } => Kind::MultipleChoice,
            Self::FreeResponse { .. } => Kind::FreeResponse,
            Self::RandomWinner { .. } => Kind::RandomWinner,
        }
    }
    #[must_use]
    pub const fn input_id(&self) -> InputId {
        match self {
            Self::MultipleChoice { input_id, .. }
            | Self::FreeResponse { input_id, .. }
            | Self::RandomWinner { input_id, .. } => *input_id,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub enum Reply {
    MultipleChoice {
        user_id: UserId,
        input_id: InputId,
    },
    FreeResponse {
        user_id: UserId,
        response: Vec<(InputId, String)>,
    },
    RandomWinner {
        user_id: UserId,
        input_id: InputId,
    },
}

impl Reply {
    #[must_use]
    pub const fn kind(&self) -> Kind {
        match self {
            Self::MultipleChoice { .. } => Kind::MultipleChoice,
            Self::FreeResponse { .. } => Kind::FreeResponse,
            Self::RandomWinner { .. } => Kind::RandomWinner,
        }
    }
    #[must_use]
    pub const fn user_id(&self) -> UserId {
        match self {
            Self::MultipleChoice { user_id, .. }
            | Self::FreeResponse { user_id, .. }
            | Self::RandomWinner { user_id, .. } => *user_id,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub enum Output {
    MultipleChoice {},
    FreeResponse {},
    RandomWinner {},
}

impl Output {
    #[must_use]
    pub const fn kind(&self) -> Kind {
        match self {
            Self::MultipleChoice { .. } => Kind::MultipleChoice,
            Self::FreeResponse { .. } => Kind::FreeResponse,
            Self::RandomWinner { .. } => Kind::RandomWinner,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    title: String,
    description: String,
    hours: NonZeroU64,
    hide_output: bool,
    hide_users: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Poll {
    user_id: UserId,

    content: Content,
    inputs: Vec<Input>,
    replies: Vec<Reply>,

    anchor: Option<Anchor>,
    output: Option<Output>,
}

impl Poll {
    pub const ERR_SENT: Error = Error::Other("The poll has already been sent");
    pub const ERR_UNSENT: Error = Error::Other("The poll has not been sent");
    pub const ERR_KIND: Error = Error::Other("The poll is an invalid type");
    pub const ERR_ACTIVE: Error = Error::Other("The poll is still active");

    #[must_use]
    pub fn archive_req(user_id: UserId, message_id: MessageId) -> Req<Self> {
        Req::new(format!("{NAME}/{user_id}"), message_id.to_string())
    }

    pub fn anchor(&self) -> Result<Anchor> {
        self.anchor.ok_or(Self::ERR_UNSENT)
    }
    pub fn output(&self) -> Result<&Output> {
        self.output.as_ref().ok_or(Self::ERR_ACTIVE)
    }
}

impl Request for Poll {
    type Args = UserId;

    fn req(user_id: Self::Args) -> Req<Self> {
        Req::new(NAME, user_id.to_string())
    }
}
