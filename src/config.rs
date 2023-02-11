use crate::error::{Error, Result};
use std::{fs, path::Path};

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub build: BuildConfig,
    pub http: HttpConfig,
    pub watch: WatchConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct BuildConfig {
    pub out_dir: String,
    pub page_pattern: String,
    pub partials_pattern: String,
    pub post_pattern: String,
    pub style_pattern: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct HttpConfig {
    pub command: String,
    pub args: Option<Vec<String>>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WatchConfig {
    pub paths: Vec<String>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Config> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(Error::Io)?;
        toml::from_str(text.as_str()).map_err(Error::Deserialize)
    }
}
