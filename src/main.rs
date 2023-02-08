mod args;
mod compilers;
mod config;
mod error;

use crate::compilers::{CompilerOptions, HandlebarsCompiler, MarkdownCompiler, SassCompiler};
use crate::config::Config;
use crate::error::Result;

fn main() -> Result<()> {
    env_logger::init();

    let config = Config::load("config.toml").expect("Unable to read config file");
    log::debug!("{:?}", config);

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
