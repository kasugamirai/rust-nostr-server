use rust_nostr_server::WebServer;

#[tokio::main]
async fn main() {
    const PORT: u16 = 3030;
    // Initialize the logger
    env_logger::init();

    let server = WebServer::new(PORT).await;
    server.run().await;
}
