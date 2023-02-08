use std::{fmt::Display, io};

use glob::{GlobError, PatternError};

#[derive(Debug)]
pub enum Error {
    CompilerError(Box<dyn std::error::Error>),
    DeserializeError(toml::de::Error),
    GlobError(GlobError),
    IoError(io::Error),
    PatternError(PatternError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
