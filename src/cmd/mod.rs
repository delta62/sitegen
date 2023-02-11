use crate::compilers::{CompilerOptions, HandlebarsCompiler, MarkdownCompiler, SassCompiler};
use crate::config::Config;
use crate::{
    args::Args,
    error::{Error, Result},
};
use notify::{recommended_watcher, Event, EventKind, Watcher};
use std::fs;
use std::path::Path;
use tokio::process::Command;

pub fn clean(_args: &Args, config: &Config) -> Result<()> {
    std::fs::remove_dir_all(config.build.out_dir.as_str()).map_err(Error::IoError)
}

pub fn build(_args: &Args, config: &Config) -> Result<()> {
    fs::create_dir_all(config.build.out_dir.as_str()).map_err(Error::IoError)?;

    let sass_opts = CompilerOptions {
        input_pattern: config.build.style_pattern.as_str(),
        output_path: config.build.out_dir.as_str(),
    };
    SassCompiler::compile(&sass_opts).unwrap();

    let mut handlebars = HandlebarsCompiler::new();
    handlebars.add_partials(config.build.partials_pattern.as_str())?;
    handlebars.compile_all(
        config.build.page_pattern.as_str(),
        config.build.out_dir.as_str(),
    )?;

    let markdown = MarkdownCompiler::new();
    markdown.compile(
        config.build.post_pattern.as_str(),
        config.build.out_dir.as_str(),
        &handlebars,
    )
}

pub async fn serve(args: Args, config: Config) -> Result<()> {
    clean(&args, &config)?;
    build(&args, &config)?;

    let local_config = config.clone();
    let command = local_config.http.command.as_str();
    let http_server_args = local_config
        .http
        .args
        .as_ref()
        .map(|v| v.as_slice())
        .unwrap_or_default();

    let mut watcher = recommended_watcher(move |res| match res {
        Ok(Event {
            kind: EventKind::Modify(_),
            paths,
            attrs,
        }) => {
            log::info!("{:?} {:?}", attrs, paths);
            build(&args, &config).unwrap();
        }
        Ok(Event {
            kind: EventKind::Create(_),
            paths,
            attrs,
        }) => {
            log::info!("{:?} {:?}", attrs, paths);
            build(&args, &config).unwrap();
        }
        _ => {}
    })
    .unwrap();

    for path in &local_config.watch.paths {
        watcher
            .watch(Path::new(path), notify::RecursiveMode::Recursive)
            .unwrap();
    }

    let _ = Command::new(command)
        .args(http_server_args)
        .kill_on_drop(true)
        .status()
        .await
        .unwrap();

    Ok(())
}
