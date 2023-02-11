use std::{fmt::Display, io};

use glob::{GlobError, PatternError};

#[derive(Debug)]
pub enum Error {
    Compiler(Box<dyn std::error::Error>),
    Deserialize(toml::de::Error),
    Glob(GlobError),
    Io(io::Error),
    Pattern(PatternError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
