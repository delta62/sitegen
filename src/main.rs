mod args;
mod cmd;
mod compilers;
mod config;
mod error;

use crate::args::{Args, Command};
use crate::config::Config;
use clap::Parser;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::init();

    let args = Args::parse();
    log::debug!("{:?}", args);

    let config = Config::load("config.toml").expect("Unable to read config file");
    log::debug!("{:?}", config);

    let result = match args.command {
        Command::Build => cmd::build(&args, &config).await,
        Command::Clean => Ok(cmd::clean(&args, &config).await),
        Command::Serve => cmd::serve(args, config).await,
    };

    if let Err(error) = result {
        log::error!("{}", error);
    }
}
