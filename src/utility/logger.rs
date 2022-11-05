use std::{fmt::Display, io::Result, path::PathBuf};

use chrono::{DateTime, Utc};
use colored::{Color, ColoredString, Colorize};
use tokio::{
    fs::{create_dir_all, File, OpenOptions},
    io::{stdout, AsyncWriteExt},
};

pub const LOG_DIR: &str = "logs/";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Level {
    #[default]
    Info,
    Warn,
    Error,
}

impl Level {
    pub const fn to_color(self) -> Color {
        match self {
            Self::Info => Color::BrightBlue,
            Self::Warn => Color::Yellow,
            Self::Error => Color::BrightRed,
        }
    }
}

impl Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Info => "Info",
            Self::Warn => "Warn",
            Self::Error => "Error",
        };

        write!(f, "({})", text)
    }
}

#[derive(Clone, Debug)]
pub struct LogData {
    time: ColoredString,
    kind: ColoredString,
    text: String,
}

impl LogData {
    pub fn to_colored_string(&self) -> String {
        format!("{} {} {}\n", self.time, self.kind, self.text)
    }
    pub fn to_cleaned_string(&self) -> String {
        let time = self.time.clone().normal();
        let kind = self.kind.clone().normal();
        format!("{} {} {}\n", time, kind, self.text)
    }
}

#[derive(Debug)]
pub struct Logger {
    file: File,
    enabled: bool,
    store_logs: bool,
}

impl Logger {
    pub async fn new(enabled: bool, store_logs: bool) -> Result<Self> {
        let created = Utc::now();
        let filename = created.format("%y-%m-%d_%H%M%S_%6f.txt").to_string();
        let dir = PathBuf::from(LOG_DIR);

        create_dir_all(&dir).await?;

        let path = dir.join(filename);
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(path)
            .await?;

        Ok(Self {
            file,
            enabled,
            store_logs,
        })
    }

    fn __filename(time: &DateTime<Utc>) -> String {
        time.format("%y-%m-%d_%H%M%S_%6f.txt").to_string()
    }
    fn __timestamp(time: &DateTime<Utc>) -> String {
        time.format("[%y-%m-%d %H:%M:%S:%3f]").to_string()
    }
    fn __parse(time: &DateTime<Utc>, level: Level, content: &str) -> LogData {
        LogData {
            time: Self::__timestamp(time).dimmed(),
            kind: level.to_string().color(level.to_color()),
            text: content.trim().to_string(),
        }
    }

    async fn __print(data: LogData) -> Result<()> {
        stdout()
            .write_all(data.to_colored_string().as_bytes())
            .await
    }
    async fn __store(file: &mut File, data: LogData) -> Result<()> {
        file.write_all(data.to_cleaned_string().as_bytes()).await
    }
    async fn __log(&mut self, time: &DateTime<Utc>, level: Level, content: &str) -> Result<()> {
        let data = Self::__parse(time, level, content);

        if self.enabled {
            if self.store_logs {
                Self::__store(&mut self.file, data.clone()).await?;
            }

            Self::__print(data).await?;
        }

        Ok(())
    }

    pub async fn info<T: ToString + Send + Sync>(&mut self, v: T) -> Result<()> {
        self.__log(&Utc::now(), Level::Info, &v.to_string()).await
    }
    pub async fn warn<T: ToString + Send + Sync>(&mut self, v: T) -> Result<()> {
        self.__log(&Utc::now(), Level::Warn, &v.to_string()).await
    }
    pub async fn error<T: ToString + Send + Sync>(&mut self, v: T) -> Result<()> {
        self.__log(&Utc::now(), Level::Error, &v.to_string()).await
    }
}
