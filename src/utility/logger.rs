use std::{
    fs::{create_dir_all, File},
    io::{stdout, Write},
    path::PathBuf,
};

use colored::{Color, Colorize};

use crate::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub const fn color(self) -> Color {
        match self {
            Self::Info => Color::BrightBlue,
            Self::Warn => Color::Yellow,
            Self::Error => Color::BrightRed,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({self:?})")
    }
}

#[derive(Clone, Debug)]
pub struct Log<'log> {
    pub level: LogLevel,
    pub time: String,
    pub text: &'log str,
}

impl<'log> Log<'log> {
    pub fn new(level: LogLevel, text: &'log str) -> Self {
        let now = Local::now();
        let time = now.format("[%y-%m-%d %H:%M:%S.%3f]").to_string();

        Self { level, time, text }
    }

    pub fn to_colored_string(&self) -> String {
        let time = self.time.dimmed();
        let kind = self.level.to_string().color(self.level.color());
        let text = self.text.trim().white();

        format!("{time} {kind} {text}\n")
    }
}

impl std::fmt::Display for Log<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = self.level.to_string();
        let text = self.text.trim();

        writeln!(f, "{} {kind} {text}", self.time)
    }
}

#[derive(Debug)]
pub struct Logger {
    path: PathBuf,
    store: bool,
    enable: bool,
}

impl Logger {
    pub const DIR: &str = "logs";
    pub const EXT: &str = "txt";

    pub fn new(store: bool, enable: bool) -> Result<Self> {
        let now = Local::now();
        let name = now.format("%y%m%d_%H%M%S_%6f").to_string();
        let dir = PathBuf::from(Self::DIR);

        create_dir_all(&dir)?;

        let path = dir.join(name).with_extension(Self::EXT);

        if store {
            File::create(&path)?;
        }

        Ok(Self {
            path,
            store,
            enable,
        })
    }

    fn __log(&self, level: LogLevel, text: &str) -> Result<()> {
        let log = Log::new(level, text);

        if self.enable {
            stdout().write_all(log.to_colored_string().as_bytes())?;

            if self.store {
                let mut file = File::options().append(true).open(&self.path)?;

                file.write_all(log.to_string().as_bytes())?;
            }
        }

        Ok(())
    }

    pub fn info(&self, s: impl Into<String>) -> Result<()> {
        self.__log(LogLevel::Info, &s.into())
    }
    pub fn warn(&self, s: impl Into<String>) -> Result<()> {
        self.__log(LogLevel::Warn, &s.into())
    }
    pub fn error(&self, s: impl Into<String>) -> Result<()> {
        self.__log(LogLevel::Error, &s.into())
    }
}
