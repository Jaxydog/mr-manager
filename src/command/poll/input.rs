use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Input {
    MultipleChoice(StandardInput),
    RandomRaffle,
    TextResponse(ModalInput),
}

impl Input {
    pub const fn kind(&self) -> Kind {
        match self {
            Self::MultipleChoice(_) => Kind::MultipleChoice,
            Self::RandomRaffle => Kind::RandomRaffle,
            Self::TextResponse(_) => Kind::TextResponse,
        }
    }

    const fn __cm_name(kind: Kind) -> &'static str {
        match kind {
            Kind::MultipleChoice => CM_CHOICE,
            Kind::RandomRaffle => CM_RAFFLE,
            Kind::TextResponse => CM_TEXT,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StandardInput {
    pub label: String,
    pub emoji: ReactionType,
}

impl AsButton for StandardInput {
    type Args<'a> = (Kind, UserId, usize);

    fn as_button(&self, disabled: bool, (kind, user, index): Self::Args<'_>) -> CreateButton {
        let custom_id = CustomId::new(Input::__cm_name(kind)).arg(user).arg(index);

        CreateButton::new(custom_id)
            .disabled(disabled)
            .emoji(self.emoji.clone())
            .label(&self.label)
            .style(ButtonStyle::Secondary)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModalInput {
    pub label: String,
    pub placeholder: Option<String>,
}
