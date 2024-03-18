mod db;
mod web;
pub use web::WebServer;
mod msgapi;
pub use db::Handlers;
pub use db::Server;
pub use msgapi::HandlerResult;
pub use msgapi::IncomingMessage;
pub use msgapi::MessageHandler;

pub enum Error {
    MsgApi(msgapi::Error),
    Server(db::Error),
    Web(web::Error),
}
