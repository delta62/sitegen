use crate::error::{Error, Result};
use std::{fs, path::Path, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    out_dir: String,
    page_dir: String,
    post_dir: String,
    static_dir: String,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Config> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(Error::IoError)?;
        toml::from_str(text.as_str()).map_err(Error::DeserializeError)
    }

    pub fn page_path<P: AsRef<Path>>(&self, cwd: P) -> PathBuf {
        let path = self.page_dir.as_str();
        cwd.as_ref().join(path)
    }

    pub fn post_path<P: AsRef<Path>>(&self, cwd: P) -> PathBuf {
        let path = self.post_dir.as_str();
        cwd.as_ref().join(path)
    }

    pub fn output_path<C, P>(&self, cwd: C, path: P) -> PathBuf
    where
        C: AsRef<Path>,
        P: AsRef<Path>,
    {
        cwd.as_ref().join(self.out_dir.as_str()).join(path)
    }
}
