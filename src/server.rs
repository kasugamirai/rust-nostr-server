use crate::Message;
use std::fmt;

use crate::Msg;
use async_trait::async_trait;
use nostr::{message::MessageHandleError, Event, RawRelayMessage, RelayMessage};
pub enum Error {
    Event(nostr::event::Error),
    MessageHandleError(MessageHandleError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Event(e) => write!(f, "event: {}", e),
            Error::MessageHandleError(e) => write!(f, "message handle error: {}", e),
        }
    }
}

impl From<nostr::event::Error> for Error {
    fn from(e: nostr::event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<MessageHandleError> for Error {
    fn from(e: MessageHandleError) -> Self {
        Self::MessageHandleError(e)
    }
}

pub struct Server {
    event: Event,
}

impl Server {
    pub async fn new(msg: String) -> Result<Self, Error> {
        let m = Msg::new(msg).await;
        let event = m.decode().await?;
        // TODO: Use the event for something
        Ok(Self { msg })
    }
}
#[async_trait]
pub trait Handlers {
    async fn req(&self) -> Result<RelayMessage, Error>;
}

#[async_trait]
impl Handlers for Server {
    async fn req(&self) -> Result<(), Error> {}
}
