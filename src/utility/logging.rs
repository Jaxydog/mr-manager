use std::{
    fs::{create_dir_all, File},
    io::{stdout, Write},
    path::PathBuf,
};

use colored::{Color, ColoredString, Colorize};

use crate::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum Level {
    Info,
    Warn,
    Error,
}

impl Level {
    #[must_use]
    pub const fn color(self) -> Color {
        match self {
            Self::Info => Color::BrightBlue,
            Self::Warn => Color::Yellow,
            Self::Error => Color::BrightRed,
        }
    }
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({self:?})")
    }
}

#[derive(Debug)]
pub struct Log {
    time: ColoredString,
    kind: ColoredString,
    text: ColoredString,
}

impl Log {
    #[must_use]
    pub fn new(level: Level, text: &str) -> Self {
        let now = Utc::now();
        let time = now.format("[%y-%m-%d %H:%M:%S:%3f]").to_string().dimmed();
        let kind = level.to_string().color(level.color());
        let text = text.trim().white();

        Self { time, kind, text }
    }

    #[must_use]
    pub fn colored(&self) -> String {
        format!("{} {} {}\n", self.time, self.kind, self.text)
    }
    #[must_use]
    pub fn cleaned(&self) -> String {
        let time = self.time.clone().normal();
        let kind = self.kind.clone().normal();
        let text = self.text.clone().normal();

        format!("{} {} {}\n", time, kind, text)
    }
}

#[derive(Debug)]
pub struct Logger {
    path: PathBuf,
    store: bool,
    enable: bool,
}

impl Logger {
    const __ROOT_DIR: &str = "logs";
    const __FILE_EXT: &str = "txt";

    pub fn new(store: bool, enable: bool) -> Result<Self> {
        let now = Utc::now();
        let name = now.format("%y%m%d_%H%M%S_%6f").to_string();
        let dir = PathBuf::from(Self::__ROOT_DIR);

        create_dir_all(&dir)?;

        let path = dir.join(name).with_extension(Self::__FILE_EXT);

        if store {
            File::create(&path)?;
        }

        Ok(Self {
            path,
            store,
            enable,
        })
    }

    fn __log(&mut self, level: Level, text: &str) -> Result<()> {
        let log = Log::new(level, text);

        if self.enable {
            stdout().write_all(log.colored().as_bytes())?;

            if self.store {
                let mut file = File::options().append(true).open(&self.path)?;
                file.write_all(log.cleaned().as_bytes())?;
            }
        }

        Ok(())
    }

    pub fn info<T: ToString>(&mut self, v: &T) -> Result<()> {
        self.__log(Level::Info, &v.to_string())
    }
    pub fn warn<T: ToString>(&mut self, v: &T) -> Result<()> {
        self.__log(Level::Warn, &v.to_string())
    }
    pub fn error<T: ToString>(&mut self, v: &T) -> Result<()> {
        self.__log(Level::Error, &v.to_string())
    }
}
