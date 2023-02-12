use crate::compilers::{CompilerOptions, HandlebarsCompiler, MarkdownCompiler, SassCompiler};
use crate::config::Config;
use crate::{
    args::Args,
    error::{Error, Result},
};
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use notify::{recommended_watcher, Event, EventKind, Watcher};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::fs;
use tokio::net::{TcpListener, TcpStream};
use tokio::process::Command;
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

type WsWriter = SplitSink<WebSocketStream<TcpStream>, Message>;
type Clients = Arc<Mutex<HashMap<String, WsWriter>>>;

pub async fn clean(_args: &Args, config: &Config) -> Result<()> {
    Ok(fs::remove_dir_all(config.build.out_dir.as_str())
        .await
        .unwrap_or_default())
}

pub async fn build(args: &Args, config: &Config) -> Result<()> {
    log::info!("build");
    fs::create_dir_all(config.build.out_dir.as_str())
        .await
        .map_err(Error::Io)?;

    let meta_path = Path::new(config.build.out_dir.as_str()).join("sitegen_meta.toml");
    log::info!("meta path {:?}", meta_path);
    fs::write(meta_path.as_path(), format!("mode={}\n", args.mode))
        .await
        .map_err(Error::Io)?;

    log::info!("meta written");

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

    let markdown = MarkdownCompiler::new();
    markdown
        .compile(
            config.build.post_pattern.as_str(),
            config.build.out_dir.as_str(),
            &handlebars,
        )
        .await
}

fn rebuild<P: AsRef<Path>>(rt: &tokio::runtime::Runtime, path: P, args: &Args, config: &Config) {
    log::info!("{:?} changed", path.as_ref());

    rt.block_on(async {
        build(&args, &config).await.unwrap();
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
    let addr = stream.peer_addr().unwrap();
    log::info!("{} connected", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
    let (writer, _) = ws_stream.split();
    let mut guard = clients.lock().unwrap();
    guard.insert(addr.to_string(), writer);
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

    let ws_clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let local_config = config.clone();
    let http_server_cmd = local_config.http.command.as_str();
    let http_server_args = local_config.http.args.as_deref().unwrap_or_default();

    let watcher_clients = ws_clients.clone();
    let mut watcher = recommended_watcher(move |res| match res {
        Ok(Event {
            kind: EventKind::Modify(_),
            paths,
            ..
        })
        | Ok(Event {
            kind: EventKind::Create(_),
            paths,
            ..
        }) => {
            let first_path = paths.first().unwrap();
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
        _ => {}
    })
    .unwrap();

    for path in &local_config.watch.paths {
        watcher
            .watch(Path::new(path), notify::RecursiveMode::Recursive)
            .expect("Unable to watch path");
    }

    // Start up a websocket server for live reloading
    let ws = ws_listen(ws_clients.clone());
    let http = http_listen(http_server_cmd, http_server_args);

    tokio::join!(ws, http);

    Ok(())
}
