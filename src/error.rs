use std::{fmt::Display, io};

use glob::{GlobError, PatternError};

#[derive(Debug)]
pub enum Error {
    Toml(toml::de::Error),
    Glob(GlobError),
    Io(io::Error),
    MarkdownError(String),
    MissingFrontMatter,
    Pattern(PatternError),
    Sass(Box<grass::Error>),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Glob(error) => write!(f, "{}", error),
            Self::Io(error) => write!(f, "{}", error),
            Self::MarkdownError(error) => write!(f, "{}", error),
            Self::MissingFrontMatter => write!(f, "missing front matter"),
            Self::Pattern(error) => write!(f, "{}", error),
            Self::Sass(error) => write!(f, "{}", error),
            Self::Toml(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
