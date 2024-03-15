use crate::IncomingMessage;
use crate::MessageHandler;
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
}

impl WebServer {
    pub async fn new(port: u16) -> Self {
        let addr = format!("127.0.0.1:{}", port)
            .parse()
            .expect("Invalid address");
        WebServer { addr }
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
        let msg_handler = IncomingMessage::new()
            .await
            .expect("Failed to create message handler");

        let (mut write, mut read) = ws_stream.split();
        while let Some(message) = read.next().await {
            match message {
                Ok(msg) => match msg {
                    Message::Text(txt) => {
                        let m = msg_handler.to_client_message(&txt).await.unwrap();
                        let msgs = msg_handler.handlers(m).await.unwrap();
                        for msg in msgs {
                            let message = Message::Text(msg);
                            self.echo_message(&mut write, &message).await;
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
