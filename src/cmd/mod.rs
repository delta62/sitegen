use crate::compilers::{CompilerOptions, HandlebarsCompiler, MarkdownCompiler, SassCompiler};
use crate::config::Config;
use crate::{
    args::Args,
    error::{Error, Result},
};
use notify::{recommended_watcher, Event, EventKind, Watcher};
use std::path::Path;
use tokio::fs;
use tokio::process::Command;

pub async fn clean(_args: &Args, config: &Config) -> Result<()> {
    fs::remove_dir_all(config.build.out_dir.as_str())
        .await
        .map_err(Error::Io)
}

pub async fn build(_args: &Args, config: &Config) -> Result<()> {
    fs::create_dir_all(config.build.out_dir.as_str())
        .await
        .map_err(Error::Io)?;

    let sass_opts = CompilerOptions {
        input_pattern: config.build.style_pattern.as_str(),
        output_path: config.build.out_dir.as_str(),
    };
    let sass_compiler = SassCompiler::new(sass_opts);
    sass_compiler.compile().await?;

    let mut handlebars = HandlebarsCompiler::new();
    handlebars
        .add_partials(config.build.partials_pattern.as_str())
        .await?;
    handlebars
        .compile_all(
            config.build.page_pattern.as_str(),
            config.build.out_dir.as_str(),
        )
        .await?;

    let markdown = MarkdownCompiler::new();
    markdown
        .compile(
            config.build.post_pattern.as_str(),
            config.build.out_dir.as_str(),
            &handlebars,
        )
        .await
}

pub async fn serve(args: Args, config: Config) -> Result<()> {
    clean(&args, &config).await?;
    build(&args, &config).await?;

    let local_config = config.clone();
    let command = local_config.http.command.as_str();
    let http_server_args = local_config.http.args.as_deref().unwrap_or_default();

    let mut watcher = recommended_watcher(move |res| match res {
        Ok(Event {
            kind: EventKind::Modify(_),
            paths,
            attrs,
        }) => {
            log::info!("{:?} {:?}", attrs, paths);
            build(&args, &config);
        }
        Ok(Event {
            kind: EventKind::Create(_),
            paths,
            attrs,
        }) => {
            log::info!("{:?} {:?}", attrs, paths);
            build(&args, &config);
        }
        _ => {}
    })
    .unwrap();

    for path in &local_config.watch.paths {
        watcher
            .watch(Path::new(path), notify::RecursiveMode::Recursive)
            .expect("Unable to watch path");
    }

    let _ = Command::new(command)
        .args(http_server_args)
        .kill_on_drop(true)
        .status()
        .await
        .unwrap();

    Ok(())
}
