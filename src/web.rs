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
            let handle_conn = WebServer::handle_connection(stream);
            tokio::spawn(handle_conn);
        }
    }

    // Note the removal of `&self` from the method signature
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
                Ok(msg) => {
                    // Handle message
                    log::error!("Received message: {}", msg);
                }
                Err(e) => {
                    log::error!("WebSocket handler failed: {}", e);
                    return;
                }
            }
        }
    }
}
