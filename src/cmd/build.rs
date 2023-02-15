use crate::args::Args;
use crate::compilers::{
    CompilerOptions, FileCopier, HandlebarsCompiler, MarkdownCompiler, SassCompiler,
};
use crate::config::Config;
use crate::error::{Error, Result};
use std::path::Path;
use tokio::fs;

pub async fn build(args: &Args, config: &Config) -> Result<()> {
    fs::create_dir_all(config.build.out_dir.as_str())
        .await
        .map_err(Error::Io)?;

    let meta_path = Path::new(config.build.out_dir.as_str()).join("sitegen_meta.toml");
    fs::write(meta_path.as_path(), format!("mode = {}\n", args.mode))
        .await
        .map_err(Error::Io)?;

    let sass_opts = CompilerOptions {
        input_pattern: config.build.style_pattern.as_str(),
        output_path: config.build.out_dir.as_str(),
    };
    let sass_compiler = SassCompiler::new(sass_opts);
    sass_compiler.compile().await?;

    let mut handlebars = HandlebarsCompiler::new(args.mode);
    handlebars
        .add_partials(config.build.partials_pattern.as_str())
        .await?;
    handlebars
        .compile_all(
            config.build.page_pattern.as_str(),
            config.build.out_dir.as_str(),
        )
        .await?;

    let markdown = MarkdownCompiler::new(args.mode);
    markdown
        .compile(
            config.build.post_pattern.as_str(),
            config.build.out_dir.as_str(),
            &handlebars,
        )
        .await?;

    let file_copy = FileCopier::new(config.build.copy.as_ref(), config.build.out_dir.as_str());
    file_copy.copy().await?;

    log::info!("Build complete");

    Ok(())
}
