use crate::IncomingMessage;
use crate::MessageHandler;
//use crate::{Handlers, Server};
use async_trait::async_trait;
use futures_util::stream::SplitSink;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use log::debug;
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
use serde_json::de;
use serde_json::json;
use std::fmt;
use std::net::SocketAddr;
use tokio_tungstenite::WebSocketStream;
//use std::os::macos::raw;
use crate::HandlerResult;
use std::sync::Arc;
use std::thread::sleep;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::http::response;
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Debug)]
pub enum Error {
    Event(nostr::event::Error),
    MessageHandle(MessageHandleError),
    Database(DatabaseError),
    TcpListener(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Event(e) => write!(f, "event: {}", e),
            Error::MessageHandle(e) => write!(f, "message handle error: {}", e),
            Error::Database(e) => write!(f, "database error: {}", e),
            Error::TcpListener(e) => write!(f, "tcp listener error: {}", e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::TcpListener(e)
    }
}

impl From<nostr::event::Error> for Error {
    fn from(e: nostr::event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<MessageHandleError> for Error {
    fn from(e: MessageHandleError) -> Self {
        Self::MessageHandle(e)
    }
}

impl From<DatabaseError> for Error {
    fn from(e: DatabaseError) -> Self {
        Self::Database(e)
    }
}

#[derive(Clone, Debug)]
pub struct WebServer {
    addr: SocketAddr,
    handler: IncomingMessage,
}

impl fmt::Display for WebServer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WebServer at {}, handler: {:?}", self.addr, self.handler)
    }
}

impl WebServer {
    pub async fn new(port: u16) -> Self {
        let addr = format!("127.0.0.1:{}", port)
            .parse()
            .expect("Invalid address");
        let handler = IncomingMessage::new()
            .await
            .expect("Failed to create message handler");
        debug!("WebServer created at {}", addr);
        //debug!("Message handler created: {:?}", handler);
        Self { addr, handler }
    }

    pub async fn start_listening(&self) -> Result<TcpListener, Error> {
        let listener = TcpListener::bind(&self.addr).await?;
        println!("WebSocket server running at {}", self.addr);
        Ok(listener)
    }

    pub async fn accept_connection(&self, listener: TcpListener) {
        while let Ok((stream, _)) = listener.accept().await {
            let this = self.clone();
            tokio::spawn(async move {
                this.handle_connection(stream).await;
            });
        }
    }

    pub async fn run(&self) {
        match self.start_listening().await {
            Ok(l) => {
                self.accept_connection(l).await;
            }
            Err(e) => {
                log::error!("Failed to start listening: {}", e);
            }
        };
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
}

pub trait Conn {
    async fn handle_connection(&self, stream: TcpStream);
    async fn close_connection(
        &self,
        write: &mut futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
            tokio_tungstenite::tungstenite::protocol::Message,
        >,
    );
    async fn handle_result(
        &self,
        result: HandlerResult,
        write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    );
}

impl Conn for WebServer {
    async fn close_connection(
        &self,
        write: &mut futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
            tokio_tungstenite::tungstenite::protocol::Message,
        >,
    ) {
        let close_message = tokio_tungstenite::tungstenite::protocol::Message::Close(Some(
            tokio_tungstenite::tungstenite::protocol::CloseFrame {
                code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Normal,
                reason: "".into(),
            },
        ));

        // Send the close message to the client.
        if let Err(e) = write.send(close_message).await {
            log::error!("Failed to send close message: {}", e);
        }

        // Close the underlying WebSocket stream.
        if let Err(e) = write.close().await {
            log::error!("Failed to close WebSocket stream: {}", e);
        }
    }

    async fn handle_connection(&self, stream: TcpStream) {
        let ws_stream: tokio_tungstenite::WebSocketStream<TcpStream> =
            match accept_async(stream).await {
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
                        let m = self.handler.to_client_message(&txt).await.unwrap();
                        let results = self.handler.handlers(m).await.unwrap();
                        self.handle_result(results, &mut write).await;
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

    async fn handle_result(
        &self,
        result: HandlerResult,
        mut write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    ) {
        match result {
            HandlerResult::String(msg) => {
                let message = Message::Text(msg);
                self.echo_message(&mut write, &message).await;
                self.close_connection(&mut write).await;
            }
            HandlerResult::Strings(msgs) => {
                for msg in msgs {
                    let message = Message::Text(msg);
                    self.echo_message(&mut write, &message).await;
                    self.close_connection(&mut write).await;
                }
            }
        }
    }
}
