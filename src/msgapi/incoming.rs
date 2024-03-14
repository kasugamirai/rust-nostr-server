use async_trait::async_trait;
use nostr::{Event, JsonUtil, PartialEvent, SECP256K1};
use std::{fmt, result};

pub enum Error {
    Event(nostr::event::Error),
    CheckSignature(nostr::event::partial::Error),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Event(e) => write!(f, "{}", e),
            Error::CheckSignature(e) => write!(f, "{}", e),
        }
    }
}

impl From<nostr::event::Error> for Error {
    fn from(e: nostr::event::Error) -> Self {
        Self::Event(e)
    }
}

impl From<nostr::event::partial::Error> for Error {
    fn from(e: nostr::event::partial::Error) -> Self {
        Self::CheckSignature(e)
    }
}

pub struct Msg {
    message: String,
}

impl Msg {
    pub async fn new(msg: String) -> Self {
        Msg { message: msg }
    }
}

#[async_trait]
pub trait IncomingMessage {
    async fn decode(&self) -> Result<Event, Error>;
    async fn is_event(&self, msg: &str) -> Result<bool, Error>;
    async fn convert_to_partial(&self) -> Result<PartialEvent, Error>;
    async fn check_signature_ctx(&self) -> result::Result<bool, Error>;
    async fn check_signature(&self) -> result::Result<bool, Error>;
    async fn exists(&self) -> bool;
}

#[async_trait]
impl IncomingMessage for Msg {
    async fn decode(&self) -> Result<Event, Error> {
        let event = Event::from_json(&self.message).map_err(Error::Event)?;
        Ok(event)
    }

    async fn is_event(&self, msg: &str) -> Result<bool, Error> {
        match Event::from_json(&msg) {
            Ok(_) => Ok(true),
            Err(e) => Err(Error::Event(e)),
        }
    }

    async fn convert_to_partial(&self) -> Result<PartialEvent, Error> {
        let event = self.decode().await?;
        let partial_event = PartialEvent {
            id: event.id,
            pubkey: event.pubkey,
            sig: event.sig,
        };

        Ok(partial_event)
    }

    async fn check_signature_ctx(&self) -> result::Result<bool, Error> {
        let partial_event = self.convert_to_partial().await?;
        PartialEvent::verify_signature_with_ctx(&partial_event, &SECP256K1)?;
        Ok(true)
    }

    async fn check_signature(&self) -> result::Result<bool, Error> {
        let partial_event = self.convert_to_partial().await?;
        PartialEvent::verify_signature(&partial_event)?;
        Ok(true)
    }

    async fn exists(&self) -> bool {
        //TODO: Implement this
        false
    }
}
