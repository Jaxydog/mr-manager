use crate::prelude::*;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimeFlag {
    ShortTime,
    LongTime,
    ShortDate,
    LongDate,
    ShortDateTime,
    LongDateTime,
    #[default]
    Relative,
}

impl Display for TimeFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let flag = match self {
            Self::ShortTime => 't',
            Self::LongTime => 'T',
            Self::ShortDate => 'd',
            Self::LongDate => 'D',
            Self::ShortDateTime => 'f',
            Self::LongDateTime => 'F',
            Self::Relative => 'R',
        };

        write!(f, "{flag}")
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimeString(i64, Option<TimeFlag>);

impl TimeString {
    pub const fn new(ms: i64) -> Self {
        Self(ms, None)
    }
    pub const fn flag(mut self, flag: TimeFlag) -> Self {
        self.1 = Some(flag);
        self
    }
}

impl From<DateTime<Utc>> for TimeString {
    fn from(value: DateTime<Utc>) -> Self {
        Self::new(value.timestamp_millis())
    }
}

impl Display for TimeString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = self.0 / 1000;
        let k = self.1.unwrap_or_default();

        write!(f, "<t:{n}:{k}>")
    }
}
