mod args;
mod config;
mod error;

use std::fs;

use crate::args::Args;
use crate::config::Config;
use crate::error::{Error, Result};
use clap::Parser;

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    let cwd = args.cwd()?;
    let cfg_path = cwd.join("config.toml");
    let config = Config::load(cfg_path).expect("Unable to read config file");

    let page_path = config.page_path(&cwd);
    let pages = fs::read_dir(&page_path).map_err(Error::IoError)?;

    for page in pages {
        let page = page.map_err(Error::IoError)?;
        let path = page.path();
        let path = path.as_path().strip_prefix(&page_path).unwrap();
        let path = config.output_path(&cwd, path);

        log::info!("{:?} -> {:?}", page.path(), path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::copy(page.path(), path).unwrap();
    }

    Ok(())
}
