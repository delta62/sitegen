use std::fmt::Display;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum BuildMode {
    /// Non-optimized build with devtools support
    Development,
    /// Optimized build without any extra functionality
    Release,
}

impl Display for BuildMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Development => "development",
            Self::Release => "release",
        };

        write!(f, "{}", s)
    }
}

#[derive(Clone, Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(value_enum)]
    pub command: Command,

    /// The mode to build the site in
    #[arg(value_enum, short, long, default_value_t = BuildMode::Development)]
    pub mode: BuildMode,
}
