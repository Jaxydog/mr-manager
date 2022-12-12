use super::*;

pub const BUTTON_CHOICE: &str = formatcp!("{NAME}_choice");
pub const BUTTON_RESPONSE: &str = formatcp!("{NAME}_response");
pub const BUTTON_RAFFLE: &str = formatcp!("{NAME}_raffle");

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputData<T> {
    pub label: String,
    pub data: Option<T>,
}

impl AsButton<(UserId, usize)> for InputData<ReactionType> {
    fn as_button(&self, disabled: bool, (user, index): (UserId, usize)) -> CreateButton {
        let custom_id = CustomId::new(BUTTON_CHOICE).arg(user).arg(index);
        let mut builder = CreateButton::new(custom_id)
            .disabled(disabled)
            .label(&self.label)
            .style(ButtonStyle::Secondary);

        if let Some(emoji) = self.data.clone() {
            builder = builder.emoji(emoji);
        }

        builder
    }
}

impl AsInputText<usize> for InputData<String> {
    fn as_input_text(&self, index: usize) -> CreateInputText {
        let mut builder =
            CreateInputText::new(InputTextStyle::Short, &self.label, index.to_string())
                .max_length(256)
                .required(true);

        if let Some(label) = self.data.clone() {
            builder = builder.placeholder(label);
        }

        builder
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Input {
    Choice(InputData<ReactionType>),
    Response(InputData<String>),
}

impl Input {
    pub const fn max_count(kind: Kind) -> usize {
        match kind {
            Kind::Choice => 10,
            Kind::Response => 5,
            Kind::Raffle => 0,
        }
    }
    pub fn label(&self) -> &str {
        match self {
            Input::Choice(data) => &data.label,
            Input::Response(data) => &data.label,
        }
    }
}
