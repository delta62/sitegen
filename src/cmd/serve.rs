use super::build;
use super::clean;
use super::http::http_listen;
use super::rebuild::rebuild;
use super::websocket;
use crate::cmd::websocket::WebSocketServer;
use crate::config::Config;
use crate::{args::Args, error::Result};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use std::path::Path;
use std::time::Duration;

pub async fn serve(args: Args, config: Config) -> Result<()> {
    log::info!("booting up; build_mode = {:?}", args.mode);

    clean(&args, &config).await;
    build(&args, &config).await?;

    let websocket_server = WebSocketServer::new();
    let debounce_ws = websocket_server.clone();
    let debounce_config = config.clone();
    let timeout = Duration::from_millis(50);
    let tick_rate = None;

    let mut debouncer = new_debouncer(
        timeout,
        tick_rate,
        move |res: DebounceEventResult| match res {
            Ok(events) => {
                let first_path = &events.first().unwrap().path;
                let rt = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .unwrap();

                let build_result = rebuild(&rt, first_path, &args, &debounce_config);
                if let Err(error) = build_result {
                    log::error!("{}", error);
                    return;
                }

                websocket::reload_all(rt, &debounce_ws);
            }
            Err(_) => {}
        },
    )
    .unwrap();

    for path in &config.watch.paths {
        debouncer
            .watcher()
            .watch(Path::new(path), notify::RecursiveMode::Recursive)
            .expect("Unable to watch path");
    }
    log::info!("Watching for changes");

    let http_server_cmd = config.http.command.as_str();
    let http_server_args = config.http.args.as_deref().unwrap_or_default();

    let ws = websocket::listen("localhost:8081", &websocket_server);
    let http = http_listen(http_server_cmd, http_server_args);
    log::info!("Webserver running");

    let (result, _) = tokio::join!(ws, http);
    result
}
