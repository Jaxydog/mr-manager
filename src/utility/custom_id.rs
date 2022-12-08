use std::collections::VecDeque;

use crate::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CustomId {
    pub base: String,
    pub name: String,
    pub args: Vec<String>,
}

impl CustomId {
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(name: impl ToString) -> Self {
        let name: String = name.to_string();
        let base = name.split('_').next().unwrap_or(&name).to_string();
        let args = vec![];

        Self { base, name, args }
    }
    #[allow(clippy::needless_pass_by_value)]
    pub fn arg(mut self, arg: impl ToString) -> Self {
        self.args.push(arg.to_string());
        self
    }
}

impl TryFrom<&str> for CustomId {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self> {
        let mut parts: VecDeque<_> = value.split(';').collect();

        let name = parts
            .pop_front()
            .ok_or_else(|| Error::InvalidId(Value::CustomId, value.to_string()))?;
        let base = name
            .split('_')
            .next()
            .ok_or_else(|| Error::InvalidId(Value::CustomId, value.to_string()))?;

        Ok(Self {
            base: base.to_string(),
            name: name.to_string(),
            args: parts.into_iter().map(ToString::to_string).collect(),
        })
    }
}

impl TryFrom<String> for CustomId {
    type Error = Error;

    fn try_from(value: String) -> Result<Self> {
        <Self as TryFrom<&str>>::try_from(&value)
    }
}

impl From<CustomId> for String {
    fn from(value: CustomId) -> Self {
        value.to_string()
    }
}

impl Display for CustomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let args = self.args.join(";");

        write!(f, "{};{args}", self.name)
    }
}
