use std::{fmt, string};

use nostr::{Event, JsonUtil};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    event(nostr::event::Error),
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            self::Error::event(e) => write!(f, "event: {}", e),
        }
    }
}

impl From<nostr::event::Error> for Error {
    fn from(e: nostr::event::Error) -> Self {
        Self::event(e)
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
        let event = Event::from_json(&self.message).map_err(Error::event)?;
        Ok(event)
    }

    pub fn is_event(&self, msg: &str) -> Result<bool, Error> {
        match Event::from_json(&msg) {
            Ok(_) => Ok(true),
            Err(e) => Err(Error::event(e)),
        }
    }

    pub fn check_signature(&self) -> bool {
        //TODO: Implement this
        false
    }

    pub fn exists(&self) -> bool {
        //TODO: Implement this
        false
    }
}
