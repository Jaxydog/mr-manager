use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Reply {
    MultipleChoice(StandardReply),
    RandomRaffle(DumbReply),
    TextResponse(ModalReply),
}

impl Reply {
    pub const fn kind(&self) -> Kind {
        match self {
            Self::MultipleChoice(_) => Kind::MultipleChoice,
            Self::RandomRaffle(_) => Kind::RandomRaffle,
            Self::TextResponse(_) => Kind::TextResponse,
        }
    }
    pub const fn user(&self) -> UserId {
        match self {
            Self::MultipleChoice(i) => i.user,
            Self::RandomRaffle(i) => i.user,
            Self::TextResponse(i) => i.user,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StandardReply {
    pub user: UserId,
    pub index: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DumbReply {
    pub user: UserId,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModalReply {
    pub user: UserId,
    pub answers: Vec<String>,
}
