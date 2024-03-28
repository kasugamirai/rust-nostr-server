use crate::IncomingMessage;
use crate::RateLimiter;
use futures_util::stream::SplitSink;
use futures_util::stream::StreamExt;
use futures_util::SinkExt;
use log::debug;
use nostr::message::MessageHandleError;
use nostr::ClientMessage;
use nostr_database::DatabaseError;
use std::fmt;
use std::net::SocketAddr;
use std::time::Duration;
use tokio_tungstenite::WebSocketStream;
//use std::os::macos::raw;
use crate::HandlerResult;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

const CONNECTED: &'static str = "New WebSocket connection";
const CLOSE: &'static str = "Received close message";

#[derive(Clone, Debug)]
pub struct WebServer {
    addr: SocketAddr,
    handler: IncomingMessage,
    limiter: RateLimiter,
}

impl fmt::Display for WebServer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WebServer at {}, handler: {:?}", self.addr, self.handler)
    }
}

impl WebServer {
    pub async fn new(port: u16) -> Self {
        let address = Self::create_address(port);
        let message_handler = Self::new_handler().await;
        let rate_limiter = Self::new_rate_limiter();

        debug!("WebServer created at {}", address);
        WebServer {
            addr: address,
            handler: message_handler,
            limiter: rate_limiter,
        }
    }

    fn new_rate_limiter() -> RateLimiter {
        RateLimiter::new(120, Duration::from_secs(60))
    }

    async fn new_handler() -> IncomingMessage {
        IncomingMessage::new().await.unwrap_or_else(|e| {
            eprintln!("Failed to create message handler: {}", e);
            std::process::exit(1);
        })
    }

    fn create_address(port: u16) -> SocketAddr {
        format!("127.0.0.1:{}", port)
            .parse::<SocketAddr>()
            .unwrap_or_else(|e| {
                eprintln!("Failed to parse address: {}", e);
                std::process::exit(1);
            })
    }

    pub async fn start_listening(&self) -> Result<TcpListener, std::io::Error> {
        let listener: TcpListener = TcpListener::bind(&self.addr).await?;
        println!("WebSocket server running at {}", self.addr);
        Ok(listener)
    }

    pub async fn accept_connection(&self, listener: TcpListener) {
        while let Ok((stream, _)) = listener.accept().await {
            let this: WebServer = self.clone();
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

        let _ = write.send(close_message).await;

        if let Err(e) = write.close().await {
            log::error!("Failed to close WebSocket stream: {}", e);
        }
    }
}

pub trait Conn {
    async fn handle_connection(&self, stream: TcpStream);
    async fn handle_result(
        &self,
        result: HandlerResult,
        write: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
    );
}

impl Conn for WebServer {
    async fn handle_connection(&self, stream: TcpStream) {
        let c = Connection::new(stream).await;
        let ws_stream: tokio_tungstenite::WebSocketStream<TcpStream> =
            match accept_async(c.stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    log::error!("WebSocket handler failed: {}", e);
                    return;
                }
            };
        let limiter = &self.limiter;
        log::debug!("{}", CONNECTED);
        let (mut write, mut read) = ws_stream.split();
        while let Some(message) = read.next().await {
            if limiter.acquire().await.is_err() {
                log::error!("Rate limit exceeded");
                return;
            }
            match message {
                Ok(msg) => match msg {
                    Message::Text(txt) => {
                        let m = self.handler.to_client_message(&txt).await;
                        let m: ClientMessage = match m {
                            Ok(message) => message,
                            Err(err) => {
                                log::error!("Failed to parse message: {}", err);
                                panic!("Failed to parse message");
                            }
                        };

                        let results = self.handler.handlers(m).await;
                        let results: HandlerResult = match results {
                            Ok(result) => result,
                            Err(err) => {
                                log::error!("Failed to handle message: {}", err);
                                panic!("Failed to handle message");
                            }
                        };

                        self.handle_result(results, &mut write).await;
                    }

                    //TODO: handle binary messages
                    Message::Binary(bin) => {
                        println!("Received binary: {:?}", bin);
                    }

                    Message::Close(Some(close_frame)) => {
                        log::debug!("Received close frame: {:?}", close_frame);
                        self.close_connection(&mut write).await;
                        break;
                    }

                    Message::Close(None) => {
                        log::debug!("{}", CLOSE);
                        self.close_connection(&mut write).await;
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
                let message: Message = Message::Text(msg);
                self.echo_message(&mut write, &message).await;
                //self.close_connection(&mut write).await;
            }
            HandlerResult::Strings(msgs) => {
                for msg in msgs {
                    let message: Message = Message::Text(msg);
                    self.echo_message(&mut write, &message).await;
                    //self.close_connection(&mut write).await;
                }
            }
            HandlerResult::DoClose(do_close) => {
                let message: Message = Message::Text(do_close.get_data().await.to_string());
                self.echo_message(&mut write, &message).await;
                self.close_connection(&mut write).await;
            }
            HandlerResult::DoAuth(do_auth, _) => {
                let message: Message = Message::Text(do_auth.get_data().await.to_string());
                self.echo_message(&mut write, &message).await;
                //self.close_connection(&mut write).await;
            }
            HandlerResult::DoEvent(do_event) => {
                let message: Message = Message::Text(do_event.get_data().await.to_string());
                self.echo_message(&mut write, &message).await;
                //self.close_connection(&mut write).await;
            }
            HandlerResult::DoReq(do_req) => {
                let msgs: &Vec<String> = do_req.get_data().await;
                for msg in msgs {
                    let message: Message = Message::Text(msg.to_string());
                    self.echo_message(&mut write, &message).await;
                    //self.close_connection(&mut write).await;
                }
            }
            HandlerResult::DoCount(do_count) => {
                let message: Message = Message::Text(do_count.get_data().await.to_string());
                self.echo_message(&mut write, &message).await;
                //self.close_connection(&mut write).await;
            }
        }
    }
}

struct Connection {
    stream: TcpStream,
    authenticated: bool,
}

impl Connection {
    async fn new(stream: TcpStream) -> Self {
        Connection {
            stream,
            authenticated: false,
        }
    }
    async fn authenticate(&mut self) {
        self.authenticated = true;
    }

    async fn is_authenticated(&self) -> bool {
        self.authenticated
    }
}
