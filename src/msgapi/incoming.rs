use super::Inner::InnerReq;
use super::Inner::{self, InnerEvent};
use super::Inner::{InnerAuth, MessageHandle};
use super::Inner::{InnerClose, InnerCount};
use async_trait::async_trait;
use nostr::{
    event, message::MessageHandleError, ClientMessage, Event, RawRelayMessage, RelayMessage,
};
use nostr_database::{DatabaseError, NostrDatabase, Order};
use nostr_rocksdb::RocksDatabase;
use Inner::HandlerResult;

const DEDUPLICATED_EVENT: &str = "deduplicated event";
const EVENT_SIGNATURE_VALID: &str = "event signature is valid";

#[derive(Debug, Clone)]
pub struct IncomingMessage {
    db: RocksDatabase,
}

#[derive(Debug)]
pub enum Error {
    Event(event::Error),
    MessageHandle(MessageHandleError),
    Database(nostr_rocksdb::database::DatabaseError),
    ToClientMessage(serde_json::Error),
    InnerMessage(Inner::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Event(e) => write!(f, "event: {}", e),
            Self::MessageHandle(e) => write!(f, "message handle error: {}", e),
            Self::Database(e) => write!(f, "database error: {}", e),
            Self::ToClientMessage(e) => write!(f, "to client message error: {}", e),
            Self::InnerMessage(e) => write!(f, "inner message error: {}", e),
        }
    }
}

impl From<Inner::Error> for Error {
    fn from(e: Inner::Error) -> Self {
        Self::InnerMessage(e)
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

impl From<nostr_rocksdb::database::DatabaseError> for Error {
    fn from(e: nostr_rocksdb::database::DatabaseError) -> Self {
        Self::Database(e)
    }
}

impl From<MessageHandleError> for Error {
    fn from(e: MessageHandleError) -> Self {
        Self::MessageHandle(e)
    }
}

impl IncomingMessage {
    pub async fn new() -> Result<Self, Error> {
        let db = RocksDatabase::open("./db/rocksdb").await?;
        Ok(Self { db })
    }
}

#[async_trait]
pub trait MessageHandler {
    async fn handlers(&self, client_message: ClientMessage) -> Result<HandlerResult, Error>;
    async fn to_client_message(&self, txt: &str) -> Result<ClientMessage, Error>;
    async fn check_signature<'a>(&self, event: &'a Event) -> Result<(), Error>;
}

#[async_trait]
impl MessageHandler for IncomingMessage {
    async fn check_signature<'a>(&self, event: &'a Event) -> Result<(), Error> {
        let ret = event.verify_signature()?;
        Ok(ret)
    }
    async fn to_client_message(&self, txt: &str) -> Result<ClientMessage, Error> {
        let ret: ClientMessage = <ClientMessage as nostr::JsonUtil>::from_json(txt)?;
        Ok(ret)
    }
    async fn handlers(&self, client_message: ClientMessage) -> Result<HandlerResult, Error> {
        match client_message {
            ClientMessage::Event(event) => {
                let InnerEvent = InnerEvent::new(event, self.db.clone()).await;
                let ret = InnerEvent.handle().await?;
                Ok(ret)
            }
            ClientMessage::Auth(auth) => {
                let InnerAuth = InnerAuth::new(auth).await;
                let ret = InnerAuth.handle().await?;
                Ok(ret)
            }
            ClientMessage::Close(sid) => {
                let InnerClose = InnerClose::new(sid).await;
                let ret = InnerClose.handle().await?;
                Ok(ret)
            }
            ClientMessage::NegClose { subscription_id } => {
                //TODO
                let CloseMessage = "close message";
                let response: RelayMessage = RelayMessage::closed(subscription_id, CloseMessage);
                //let ret: DoNegClose;
                let ret: Vec<String> = vec!["TODO".to_string()];
                Ok(HandlerResult::Strings(ret))
            }
            ClientMessage::Count {
                subscription_id,
                filters,
            } => {
                let InnerCount = InnerCount::new(subscription_id, filters, self.db.clone()).await;
                let ret = InnerCount.handle().await?;
                Ok(ret)
            }
            ClientMessage::Req {
                subscription_id,
                filters,
            } => {
                let InnerReq = InnerReq::new(subscription_id, filters, self.db.clone()).await;
                let ret = InnerReq.handle().await?;
                Ok(ret)
            }
            ClientMessage::NegOpen {
                subscription_id,
                filter,
                id_size,
                initial_message,
            } => {
                let ret: Vec<String> = vec!["TODO".to_string()];
                Ok(HandlerResult::Strings(ret))
            }
            ClientMessage::NegMsg {
                subscription_id,
                message,
            } => {
                let ret = vec!["TODO".to_string()];
                Ok(HandlerResult::Strings(ret))
            }
        }
    }
}
