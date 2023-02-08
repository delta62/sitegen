use clap::{Parser, ValueEnum};

#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
pub enum Command {
    /// Build the site
    Build,
    /// Remove previously built artifacts
    Clean,
    /// Start a development server and host the site locally
    Serve,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(value_enum)]
    pub command: Command,
}
