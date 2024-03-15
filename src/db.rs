use crate::msgapi;
use nostr_database::{DatabaseError, NostrDatabase, Order};
use nostr_rocksdb::RocksDatabase;
use std::fmt;
use tokio::sync::Mutex;

use crate::msgapi::IncomingMessage;
use async_trait::async_trait;
use nostr::{
    event,
    message::{self, MessageHandleError},
    Event, RawRelayMessage, RelayMessage,
};

pub enum Error {
    Event(nostr::event::Error),
    MessageHandleError(MessageHandleError),
    MsgapiError(msgapi::Error),
    DatabaseError(DatabaseError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Event(e) => write!(f, "event: {}", e),
            Error::MessageHandleError(e) => write!(f, "message handle error: {}", e),
            Error::MsgapiError(e) => write!(f, "msgapi error: {}", e),
            Error::DatabaseError(e) => write!(f, "database error: {}", e),
        }
    }
}

impl From<DatabaseError> for Error {
    fn from(e: DatabaseError) -> Self {
        Self::DatabaseError(e)
    }
}

impl From<msgapi::Error> for Error {
    fn from(e: msgapi::Error) -> Self {
        Self::MsgapiError(e)
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
    db: Mutex<RocksDatabase>,
}

impl Server {
    pub async fn new() -> Result<Self, Error> {
        let db = RocksDatabase::open("./db/rocksdb").await?;
        Ok(Server { db: Mutex::new(db) })
    }
}
#[async_trait]
pub trait Handlers {
    async fn req(&self, message: String) -> Result<(), Error>;
}

#[async_trait]
impl Handlers for Server {
    async fn req(&self, message: String) -> Result<(), Error> {
        //let m = Msg::new(message).await;
        // let event = m.decode().await?;
        //let db = self.db.lock().await;
        //db.save_event(&event).await?;
        Ok(())
    }
}
