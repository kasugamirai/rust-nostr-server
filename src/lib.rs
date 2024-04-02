//mod db;
mod limiter;
mod web;
pub use web::WebServer;
mod msgapi;
//pub use db::Handlers;
//pub use db::Server;
pub use limiter::RateLimitError;
pub use msgapi::IncomingMessage;
pub use msgapi::OutgoingHandler;
pub use msgapi::OutgoingMessage;
pub enum Error {
    MsgApi(msgapi::Error),
    //   Server(db::Error),
    //Web(web::Error),
}

pub use limiter::RateLimiter;
pub use msgapi::Challenge;
pub use msgapi::HandlerResult;
