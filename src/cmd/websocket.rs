use crate::error::{Error, Result};
use futures_util::{stream::SplitSink, SinkExt, StreamExt, TryStreamExt};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio::runtime::Runtime;
use tokio_tungstenite::WebSocketStream;
use tungstenite::Message;

type WsWriter = SplitSink<WebSocketStream<TcpStream>, Message>;

#[derive(Clone)]
pub struct WebSocketServer {
    clients: Arc<Mutex<HashMap<String, WsWriter>>>,
}

impl WebSocketServer {
    pub fn new() -> Self {
        let clients = Arc::new(Mutex::new(HashMap::new()));
        Self { clients }
    }

    fn add_client(&self, address: impl Into<String>, stream: WsWriter) -> Result<()> {
        let mut guard = self.clients.lock().unwrap();
        guard.insert(address.into(), stream);
        Ok(())
    }

    fn remove_client(&self, address: impl Into<String>) -> Result<()> {
        let mut guard = self.clients.lock().unwrap();
        guard.remove(&address.into());
        Ok(())
    }
}

pub async fn listen(address: impl ToSocketAddrs, server: &WebSocketServer) -> Result<()> {
    let listener = TcpListener::bind(address).await.unwrap();

    loop {
        let (stream, _) = listener.accept().await.map_err(Error::Io)?;
        tokio::spawn(accept_connection(server.clone(), stream));
    }
}

pub fn reload_all(rt: Runtime, server: &WebSocketServer) {
    let mut guard = server.clients.lock().unwrap();
    for writer in guard.values_mut() {
        rt.block_on(async {
            writer
                .send(Message::Text(String::from("reload")))
                .await
                .unwrap();
        });
    }
}

async fn accept_connection(server: WebSocketServer, stream: TcpStream) -> Result<()> {
    let addr = stream.peer_addr().unwrap().to_string();
    log::debug!("{} connected", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
    let (writer, reader) = ws_stream.split();

    server.add_client(addr.clone(), writer)?;

    reader
        .try_for_each(|_msg| std::future::ready(Ok(())))
        .await
        .unwrap_or_default();

    server.remove_client(addr)
}
