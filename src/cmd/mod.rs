use crate::compilers::{CompilerOptions, HandlebarsCompiler, MarkdownCompiler, SassCompiler};
use crate::config::Config;
use crate::{
    args::Args,
    error::{Error, Result},
};
use futures_util::{future, StreamExt, TryStreamExt};
use notify::{recommended_watcher, Event, EventKind, Watcher};
use std::path::Path;
use tokio::fs;
use tokio::net::{TcpListener, TcpStream};
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

fn rebuild<P: AsRef<Path>>(path: P, args: &Args, config: &Config) {
    log::info!("{:?} changed", path.as_ref());
    let rt = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();

    rt.block_on(async {
        build(&args, &config).await.unwrap();
    });
}

async fn ws_listen() {
    let addr = "localhost:8081";
    let listener = TcpListener::bind(addr).await.unwrap();

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream.peer_addr().unwrap();
    log::info!("Client {} connected", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
    log::info!("WS connection: {}", addr);

    let (write, read) = ws_stream.split();
    read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .forward(write)
        .await
        .unwrap();
}

async fn http_listen(cmd: &str, args: &[String]) {
    Command::new(cmd)
        .args(args)
        .kill_on_drop(true)
        .status()
        .await
        .map(|_| ())
        .unwrap()
}

pub async fn serve(args: Args, config: Config) -> Result<()> {
    clean(&args, &config).await?;
    build(&args, &config).await?;

    let local_config = config.clone();
    let http_server_cmd = local_config.http.command.as_str();
    let http_server_args = local_config.http.args.as_deref().unwrap_or_default();

    let mut watcher = recommended_watcher(move |res| match res {
        Ok(Event {
            kind: EventKind::Modify(_),
            paths,
            ..
        }) => rebuild(paths.first().unwrap(), &args, &config),
        Ok(Event {
            kind: EventKind::Create(_),
            paths,
            ..
        }) => rebuild(paths.first().unwrap(), &args, &config),
        _ => {}
    })
    .unwrap();

    for path in &local_config.watch.paths {
        watcher
            .watch(Path::new(path), notify::RecursiveMode::Recursive)
            .expect("Unable to watch path");
    }

    // Start up a websocket server for live reloading
    let ws = ws_listen();
    let http = http_listen(http_server_cmd, http_server_args);

    tokio::join!(ws, http);

    Ok(())
}
