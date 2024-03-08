use rust_nostr_server::WebServer;

#[tokio::main]
async fn main() {
    // Initialize the logger
    env_logger::init();
    let server = WebServer::new(3030);
    server.run().await;
}
