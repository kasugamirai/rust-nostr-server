use crate::Msg;
use crate::{Handlers, Server};
use async_trait::async_trait;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use nostr::event::EventIntermediate;
use nostr::message::MessageHandleError;
use nostr::ClientMessage;
use nostr::JsonUtil;
use nostr::RawRelayMessage;
use nostr::RelayMessage;
use nostr_database::nostr;
use nostr_database::{DatabaseError, NostrDatabase, Order};
use nostr_rocksdb::nostr::Event;
use nostr_rocksdb::RocksDatabase;
use serde_json::json;
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

    pub async fn echo_message(
        &self,
        write: &mut futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
            tokio_tungstenite::tungstenite::protocol::Message,
        >,
        message: &Message,
    ) {
        if let Err(e) = write.send(message.clone()).await {
            log::error!("Failed to echo message: {}", e);
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
                        let client_message = ClientMessage::from_json(txt).unwrap();
                        match client_message {
                            ClientMessage::Event(event) => {
                                self.db
                                    .save_event(&event)
                                    .await
                                    .expect("Failed to insert event");
                                //let raw_client_message = Message::Text(Event::as_json(&event));
                                //let messages = vec![&raw_client_message];
                                let response = vec![
                                    "OK",
                                    "70b10f70c1318967eddf12527799411b1a9780ad9c43858f5e5fcd45486a13a5",
                                    "false",
                                    "invalid: created_at too early"
                                ];

                                let response_json = serde_json::to_string(&response).unwrap();
                                let response_message = Message::Text(response_json);
                                let messages = vec![&response_message];
                                self.echo_message(&mut write, &messages[0]).await;
                                // self.echo_message(&mut write, &messages[0]).await;
                            }
                            ClientMessage::Auth(auth) => {}
                            ClientMessage::Close(close) => {}
                            ClientMessage::NegClose { subscription_id } => {}
                            ClientMessage::Count {
                                subscription_id,
                                filters,
                            } => {}
                            ClientMessage::Req {
                                subscription_id,
                                filters,
                            } => {}
                            ClientMessage::NegOpen {
                                subscription_id,
                                filter,
                                id_size,
                                initial_message,
                            } => {}
                            ClientMessage::NegMsg {
                                subscription_id,
                                message,
                            } => {}
                        }
                    }

                    //TODO:
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
