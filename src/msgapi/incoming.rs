use std::{fmt, result, string};

use nostr::{Event, JsonUtil, PartialEvent, SECP256K1};

pub enum Error {
    Event(nostr::event::Error),
    CheckSignature(nostr::event::partial::Error),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            self::Error::Event(e) => write!(f, "event: {}", e),
            self::Error::CheckSignature(e) => write!(f, "check signature: {}", e),
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
    pub fn new() -> Msg {
        Msg {
            message: String::new(),
        }
    }
    pub fn decode(&self) -> Result<Event, Error> {
        let event = Event::from_json(&self.message).map_err(Error::Event)?;
        Ok(event)
    }

    pub fn is_event(&self, msg: &str) -> Result<bool, Error> {
        match Event::from_json(&msg) {
            Ok(_) => Ok(true),
            Err(e) => Err(Error::Event(e)),
        }
    }

    pub fn convert_to_partial(&self) -> Result<PartialEvent, Error> {
        let event = self.decode()?;
        let partial_event = PartialEvent {
            id: event.id,
            pubkey: event.pubkey,
            sig: event.sig,
        };

        Ok(partial_event)
    }

    pub fn check_signature_ctx(&self) -> result::Result<bool, Error> {
        let partial_event = self.convert_to_partial()?;
        PartialEvent::verify_signature_with_ctx(&partial_event, &SECP256K1)?;
        Ok(true)
    }

    pub fn check_signature(&self) -> result::Result<bool, Error> {
        let partial_event = self.convert_to_partial()?;
        PartialEvent::verify_signature(&partial_event)?;
        Ok(true)
    }

    pub fn exists(&self) -> bool {
        //TODO: Implement this
        false
    }
}
