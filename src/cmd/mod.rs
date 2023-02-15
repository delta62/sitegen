mod build;
mod clean;
mod http;
mod rebuild;
mod websocket;

use crate::config::Config;
use crate::{args::Args, error::Result};
pub use build::build;
pub use clean::clean;
use futures_util::SinkExt;
use http::http_listen;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
pub use rebuild::rebuild;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tungstenite::Message;
use websocket::{ws_listen, Clients};

pub async fn serve(args: Args, config: Config) -> Result<()> {
    log::info!("booting up; build_mode = {:?}", args.mode);

    clean(&args, &config).await;
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

                let build_result = rebuild(&rt, first_path, &args, &config);
                if let Err(error) = build_result {
                    log::error!("{}", error);
                    return;
                }

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
