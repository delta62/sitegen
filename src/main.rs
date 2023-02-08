mod args;
mod compilers;
mod config;
mod error;

use crate::args::{Args, Command};
use crate::compilers::{CompilerOptions, HandlebarsCompiler, MarkdownCompiler, SassCompiler};
use crate::config::Config;
use crate::error::{Error, Result};
use clap::Parser;
use std::path::Path;

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    let config = Config::load("config.toml").expect("Unable to read config file");
    log::debug!("{:?}", args);
    log::debug!("{:?}", config);

    match args.command {
        Command::Build => build(config),
        Command::Clean => clean(config.out_dir.as_str()),
    }
}

fn clean<P: AsRef<Path>>(path: P) -> Result<()> {
    std::fs::remove_dir_all(path).map_err(Error::IoError)
}

fn build(config: Config) -> Result<()> {
    let sass_opts = CompilerOptions {
        input_pattern: config.style_pattern.as_str(),
        output_path: config.out_dir.as_str(),
    };
    SassCompiler::compile(&sass_opts).unwrap();

    let mut handlebars = HandlebarsCompiler::new();
    handlebars.add_partials(config.partials_pattern.as_str())?;
    handlebars.compile_all(config.page_pattern.as_str(), config.out_dir.as_str())?;

    let markdown = MarkdownCompiler::new();
    markdown.compile(
        config.post_pattern.as_str(),
        config.out_dir.as_str(),
        &handlebars,
    )
}
