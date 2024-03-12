use crate::{Handlers, Server};
use async_trait::async_trait;
use futures_util::stream::StreamExt;
use nostr::event::EventIntermediate;
use nostr::JsonUtil;
use nostr::RelayMessage;
use nostr_database::nostr;
use nostr_database::{DatabaseError, NostrDatabase, Order};
use nostr_rocksdb::RocksDatabase;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

pub struct WebServer {
    addr: SocketAddr,
    db: RocksDatabase,
}

impl WebServer {
    pub async fn new(port: u16) -> Self {
        let addr = format!("127.0.0.1:{}", port)
            .parse()
            .expect("Invalid address");
        let db = RocksDatabase::open("./db/rocksdb")
            .await
            .expect("Failed to open database");
        WebServer { addr, db }
    }

    pub async fn run(&self) {
        let listener = TcpListener::bind(&self.addr)
            .await
            .expect("cannot bind to port");
        println!("WebSocket server running at {}", self.addr);

        while let Ok((stream, _)) = listener.accept().await {
            WebServer::handle_connection(&self, stream).await;
            //TODO:fix tokio::spawn(WebServer::handle_connection(stream));
        }
    }

    // Note the removal of `&self` from the method signature
}

pub trait Conn {
    async fn handle_connection(&self, stream: TcpStream);
}

impl Conn for WebServer {
    async fn handle_connection(&self, stream: TcpStream) {
        let ws_stream = match accept_async(stream).await {
            Ok(ws) => ws,
            Err(e) => {
                log::error!("WebSocket handler failed: {}", e);
                return;
            }
        };

        println!("New WebSocket connection");

        let (_write, mut read) = ws_stream.split();
        while let Some(message) = read.next().await {
            match message {
                Ok(msg) => match msg {
                    Message::Text(mut txt) => {
                        let relay_message =
                            RelayMessage::from_json(txt).expect("Failed to parse event");
                        if let RelayMessage::Event { event, .. } = relay_message {
                            let event_id = &event.id;
                            let saved = self
                                .db
                                .has_event_already_been_saved(event_id)
                                .await
                                .expect("Failed to save event");
                        }

                        //TODO:
                    }

                    Message::Binary(bin) => {
                        println!("Received binary: {:?}", bin);
                    }
                    Message::Close(_) => {
                        println!("Received close message");
                        break;
                    }
                    _ => {}
                },
                Err(e) => {
                    log::error!("WebSocket handler failed: {}", e);
                    return;
                }
            }
        }
    }
}
