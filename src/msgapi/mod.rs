mod incoming;
mod outgoing;
pub use incoming::HandlerResult;
pub use incoming::IncomingMessage;
pub use outgoing::OutgoingHandler;
pub use outgoing::OutgoingMessage;
#[derive(Debug)]
pub enum Error {
    Incoming(incoming::Error),
    Outgoing(outgoing::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Incoming(e) => write!(f, "incoming: {}", e),
            Self::Outgoing(e) => write!(f, "outgoing: {}", e),
        }
    }
}

mod challange;

pub use challange::Challenge;
