use crate::{Handlers, Server};
use async_trait::async_trait;
use futures_util::stream::StreamExt;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

pub struct WebServer {
    addr: SocketAddr,
}

impl WebServer {
    pub fn new(port: u16) -> Self {
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
            WebServer::handle_connection(stream).await;
            //TODO:fix tokio::spawn(WebServer::handle_connection(stream));
        }
    }

    // Note the removal of `&self` from the method signature
}

pub trait Conn {
    async fn handle_connection(stream: TcpStream);
}

impl Conn for WebServer {
    async fn handle_connection(stream: TcpStream) {
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
                        let len = txt.len();
                        if len > 0 {
                            txt.truncate(len - 1);
                        }
                        let s: Result<Server, crate::server::Error> = Server::new().await;
                        let ser = match s {
                            Ok(s) => s,
                            Err(e) => {
                                log::error!("Server failed: {}", e);
                                return;
                            }
                        };
                        let r = match ser.req(txt.clone()).await {
                            Ok(r) => {
                                println!("Server response: {:?}", txt);
                            }
                            Err(e) => {
                                log::error!("Server failed: {}", e);
                                return;
                            }
                        };
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
