use std::{fmt::Display, io};

#[derive(Debug)]
pub enum Error {
    DeserializeError(toml::de::Error),
    IoError(io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
