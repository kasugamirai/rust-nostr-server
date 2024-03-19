use nostr::event;
use nostr::RelayMessage;

#[derive(Debug)]
pub enum Error {
    Event(event::Error),
    ToClientMessage(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Event(e) => write!(f, "event: {}", e),
            Self::ToClientMessage(e) => write!(f, "to client message error: {}", e),
        }
    }
}

impl From<nostr::event::Error> for Error {
    fn from(e: nostr::event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::ToClientMessage(e)
    }
}

pub struct OutgoingMessage {}

impl OutgoingMessage {
    pub async fn new() -> Self {
        OutgoingMessage {}
    }
}

pub trait Handler {
    async fn send_challenge(&self, challenge_msg: String) -> Result<String, Error>;
}

impl Handler for OutgoingMessage {
    // Change `handler` to `Handler`
    async fn send_challenge(&self, challenge_msg: String) -> Result<String, Error> {
        let relay_message: RelayMessage = RelayMessage::auth(challenge_msg);
        let challenge_str: String = serde_json::to_string(&relay_message)?;
        Ok(challenge_str)
    }
}
