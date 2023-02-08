use crate::compilers::{CompilerOptions, HandlebarsCompiler, MarkdownCompiler, SassCompiler};
use crate::config::Config;
use crate::{
    args::Args,
    error::{Error, Result},
};

pub fn clean(_args: &Args, config: &Config) -> Result<()> {
    std::fs::remove_dir_all(config.out_dir.as_str()).map_err(Error::IoError)
}

pub fn build(_args: &Args, config: &Config) -> Result<()> {
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
