use crate::error::{Error, Result};
use std::{fs, path::Path};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub out_dir: String,
    pub page_pattern: String,
    pub partials_pattern: String,
    pub post_pattern: String,
    pub static_dir: String,
    pub style_pattern: String,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Config> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(Error::IoError)?;
        toml::from_str(text.as_str()).map_err(Error::DeserializeError)
    }
}
