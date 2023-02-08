mod args;
mod cmd;
mod compilers;
mod config;
mod error;

use crate::args::{Args, Command};
use crate::config::Config;
use crate::error::Result;
use clap::Parser;

fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    log::debug!("{:?}", args);

    let config = Config::load("config.toml").expect("Unable to read config file");
    log::debug!("{:?}", config);

    match args.command {
        Command::Build => cmd::build(&args, &config),
        Command::Clean => cmd::clean(&args, &config),
        Command::Serve => cmd::serve(&args, &config),
    }
}
