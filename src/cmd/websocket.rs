use futures_util::{stream::SplitSink, StreamExt, TryStreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

pub type Clients = Arc<Mutex<HashMap<String, WsWriter>>>;
type WsWriter = SplitSink<WebSocketStream<TcpStream>, Message>;

pub async fn ws_listen(clients: Clients) {
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
