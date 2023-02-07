use crate::error::{Error, Result};
use clap::Parser;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(short, long)]
    cwd: Option<String>,
}

impl Args {
    pub fn cwd(&self) -> Result<PathBuf> {
        let path = self.cwd.as_ref();

        if let Some(path) = path {
            Ok(PathBuf::from(path.as_str()))
        } else {
            env::current_dir().map_err(Error::IoError)
        }
    }
}
