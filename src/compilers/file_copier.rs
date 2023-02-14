use glob::glob;
use std::path::Path;
use tokio::fs::{copy, create_dir_all};

use crate::error::{Error, Result};

pub struct FileCopier<'a> {
    paths: Option<&'a Vec<String>>,
    out_dir: &'a str,
}

impl<'a> FileCopier<'a> {
    pub fn new(paths: Option<&'a Vec<String>>, out_dir: &'a str) -> Self {
        Self { out_dir, paths }
    }

    pub async fn copy(&self) -> Result<()> {
        if let Some(paths) = self.paths {
            for pattern in paths {
                let files = glob(pattern).map_err(Error::Pattern)?;
                for from in files {
                    let from = from.map_err(Error::Glob)?;
                    let to = Path::new(self.out_dir).join(from.as_path());

                    create_dir_all(to.parent().unwrap())
                        .await
                        .map_err(Error::Io)?;
                    copy(from.as_path(), to).await.map_err(Error::Io)?;
                }
            }
        }

        Ok(())
    }
}
