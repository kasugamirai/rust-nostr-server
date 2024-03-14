mod db;
mod web;
pub use web::WebServer;
mod msgapi;
pub use db::Handlers;
pub use db::Server;
pub use msgapi::IncomingMessage;
pub use msgapi::Msg;

pub enum Error {
    MsgApi(msgapi::Error),
    Server(db::Error),
}
