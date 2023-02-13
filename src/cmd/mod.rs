use crate::compilers::{CompilerOptions, HandlebarsCompiler, MarkdownCompiler, SassCompiler};
use crate::config::Config;
use crate::{
    args::Args,
    error::{Error, Result},
};
use futures_util::{stream::SplitSink, SinkExt, StreamExt, TryStreamExt};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::fs;
use tokio::net::{TcpListener, TcpStream};
use tokio::process::Command;
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

type WsWriter = SplitSink<WebSocketStream<TcpStream>, Message>;
type Clients = Arc<Mutex<HashMap<String, WsWriter>>>;

pub async fn clean(_args: &Args, config: &Config) -> Result<()> {
    fs::remove_dir_all(config.build.out_dir.as_str())
        .await
        .unwrap_or_default();

    Ok(())
}

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

    log::info!("Build complete");

    Ok(())
}

fn rebuild<P: AsRef<Path>>(rt: &tokio::runtime::Runtime, path: P, args: &Args, config: &Config) {
    log::info!("change: {}", path.as_ref().to_str().unwrap());

    rt.block_on(async {
        build(args, config).await.unwrap();
    });
}

async fn ws_listen(clients: Clients) {
    let addr = "localhost:8081";
    let listener = TcpListener::bind(addr).await.unwrap();

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream, clients.clone()));
    }
}

async fn accept_connection(stream: TcpStream, clients: Clients) {
    let addr = stream.peer_addr().unwrap().to_string();
    log::debug!("{} connected", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
    let (writer, reader) = ws_stream.split();

    {
        let mut guard = clients.lock().unwrap();
        guard.insert(addr.clone(), writer);
    }

    reader
        .try_for_each(|_msg| std::future::ready(Ok(())))
        .await
        .unwrap_or_default();

    {
        log::debug!("{} disconnected", addr);
        let mut guard = clients.lock().unwrap();
        guard.remove(&addr);
    }
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
    log::info!("booting up; build_mode = {:?}", args.mode);

    clean(&args, &config).await?;
    build(&args, &config).await?;

    let ws_clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let local_config = config.clone();
    let http_server_cmd = local_config.http.command.as_str();
    let http_server_args = local_config.http.args.as_deref().unwrap_or_default();

    let watcher_clients = ws_clients.clone();
    let mut debouncer = new_debouncer(
        Duration::from_millis(50),
        None,
        move |res: DebounceEventResult| match res {
            Ok(events) => {
                let first_path = &events.first().unwrap().path;
                let rt = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap();

                rebuild(&rt, first_path, &args, &config);

                let mut clients_guard = watcher_clients.lock().unwrap();
                clients_guard.values_mut().for_each(|writer| {
                    rt.block_on(async {
                        writer
                            .send(Message::Text(String::from("reload")))
                            .await
                            .unwrap();
                    })
                });
            }
            Err(_) => {}
        },
    )
    .unwrap();

    for path in &local_config.watch.paths {
        debouncer
            .watcher()
            .watch(Path::new(path), notify::RecursiveMode::Recursive)
            .expect("Unable to watch path");
    }
    log::info!("Watching for changes");

    // Start up a websocket server for live reloading
    let ws = ws_listen(ws_clients.clone());
    let http = http_listen(http_server_cmd, http_server_args);
    log::info!("Webserver running");

    tokio::join!(ws, http);

    Ok(())
}
