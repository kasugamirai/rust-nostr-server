use rust_nostr_server::WebServer;

#[tokio::main]
async fn main() {
    let port: u16 = 3030;
    // Initialize the logger
    env_logger::init();

    let server = WebServer::new(port).await;
    server.run().await;
}
