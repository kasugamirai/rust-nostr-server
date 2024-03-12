mod server;
mod web;
pub use web::WebServer;
mod msgapi;
pub use msgapi::Message;
pub use msgapi::Msg;
pub use server::Handlers;
pub use server::Server;

pub enum Error {
    MsgApi(msgapi::Error),
    Server(server::Error),
}
