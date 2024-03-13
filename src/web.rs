use crate::{Handlers, Server};
use async_trait::async_trait;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use nostr::event::EventIntermediate;
use nostr::JsonUtil;
use nostr::RawRelayMessage;
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

        let (mut write, mut read) = ws_stream.split();
        while let Some(message) = read.next().await {
            match message {
                Ok(msg) => match msg {
                    Message::Text(txt) => {
                        print!("Received text: {:?}", txt);
                        let raw =
                            RawRelayMessage::from_json(txt).expect("Failed to parse raw message");
                        let relay_message =
                            RelayMessage::try_from(raw).expect("Failed to parse event");
                        if let RelayMessage::Event { event, .. } = relay_message {
                            let event_id = &event.id;
                            let saved = self
                                .db
                                .has_event_already_been_saved(event_id)
                                .await
                                .expect("Failed to check if event has been saved already");
                            if !saved {
                                self.db
                                    .save_event(&event)
                                    .await
                                    .expect("Failed to save event");
                                write
                                    .send(Message::Text("ok".to_string()))
                                    .await
                                    .expect("Failed to send ok message");
                                write.close().await.expect("Failed to close WebSocket");
                                return;
                            }
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
