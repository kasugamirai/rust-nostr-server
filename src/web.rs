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
use nostr_rocksdb::nostr::{event, Event};
use nostr_rocksdb::RocksDatabase;
use serde_json::json;
use std::fmt;
use std::net::SocketAddr;
use std::os::macos::raw;
use std::thread::sleep;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::http::response;
use tokio_tungstenite::tungstenite::protocol::Message;

pub enum Error {
    Event(nostr::event::Error),
    MessageHandleError(MessageHandleError),
    DatabaseError(DatabaseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Event(e) => write!(f, "event: {}", e),
            Error::MessageHandleError(e) => write!(f, "message handle error: {}", e),
            Error::DatabaseError(e) => write!(f, "database error: {}", e),
        }
    }
}

impl From<nostr::event::Error> for Error {
    fn from(e: nostr::event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<MessageHandleError> for Error {
    fn from(e: MessageHandleError) -> Self {
        Self::MessageHandleError(e)
    }
}

impl From<DatabaseError> for Error {
    fn from(e: DatabaseError) -> Self {
        Self::DatabaseError(e)
    }
}

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
            //tokio::spawn(WebServer::handle_connection(&self, stream));
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
                        let client_message: ClientMessage =
                            ClientMessage::from_json(txt).expect("Failed to parse client message");
                        match client_message {
                            ClientMessage::Event(event) => {
                                let eid = event.id();
                                let event_existed = self
                                    .db
                                    .has_event_already_been_saved(&eid)
                                    .await
                                    .expect("Failed to check if event existed");
                                print!("Event existed: {}", event_existed);
                                self.db
                                    .save_event(&event)
                                    .await
                                    .expect("Failed to insert event");

                                //let raw_client_message = Message::Text(Event::as_json(&event));
                                //let messages = vec![&raw_client_message];
                                let ids = event.id().to_string();
                                let context = event.content().to_string();
                                let response = vec!["OK", &ids, "true", &context];

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
                            } => {
                                let order = Order::Desc;
                                let queried_events = self.db.query(filters, order).await.unwrap();
                                for event in &queried_events {
                                    let raw = Event::as_json(&event);
                                    let event_json: serde_json::Value =
                                        serde_json::from_str(&raw).unwrap();
                                    let response = json!(["EVENT", "test", event_json]);
                                    let response_message = Message::Text(response.to_string());
                                    self.echo_message(&mut write, &response_message).await;
                                }
                            }
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
