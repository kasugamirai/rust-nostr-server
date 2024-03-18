use std::iter::Successors;

use async_trait::async_trait;
use futures_util::future::ok;
use nostr::JsonUtil;
use nostr::{
    event, message::MessageHandleError, ClientMessage, Event, RawRelayMessage, RelayMessage,
};
use nostr_database::nostr;
use nostr_database::{DatabaseError, NostrDatabase, Order};
use nostr_rocksdb::RocksDatabase;
use tokio_tungstenite::tungstenite::http::response;

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
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Event(e) => write!(f, "event: {}", e),
            Self::MessageHandle(e) => write!(f, "message handle error: {}", e),
            Self::Database(e) => write!(f, "database error: {}", e),
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

pub enum HandlerResult {
    Strings(Vec<String>),
    String(String),
}

#[async_trait]
pub trait MessageHandler {
    async fn handlers(&self, client_message: ClientMessage) -> Result<HandlerResult, Error>;
    async fn to_client_message(&self, txt: &str) -> Result<ClientMessage, Error>;
    async fn check_signature(&self, event: Event) -> Result<(), Error>;
}

#[async_trait]
impl MessageHandler for IncomingMessage {
    async fn check_signature(&self, event: Event) -> Result<(), Error> {
        let ret = event.verify_signature()?;
        Ok(ret)
    }
    async fn to_client_message(&self, txt: &str) -> Result<ClientMessage, Error> {
        let ret = <ClientMessage as nostr::JsonUtil>::from_json(txt)?;
        Ok(ret)
    }
    async fn handlers(&self, client_message: ClientMessage) -> Result<HandlerResult, Error> {
        match client_message {
            ClientMessage::Event(event) => {
                let response;
                let eid = event.id();
                let ids = event.id().to_string();

                match self.check_signature(*event.clone()).await {
                    Ok(_) => {
                        // If the check_signature method was not successful
                        log::info!("Event signature is valid");
                    }
                    Err(e) => {
                        let err = e.to_string();
                        // If the check_signature method returned an error, handle it here
                        response = vec!["OK", &ids, "false", &err];
                        let response_str = serde_json::to_string(&response)?;
                        return Ok(HandlerResult::String(response_str));
                    }
                }
                let content = event.content().to_string();
                let event_existed = self.db.has_event_already_been_saved(&eid).await?;
                if !event_existed {
                    let success = self.db.save_event(&event).await?;
                    if success {
                        response = vec!["OK", &ids, "true", &content];
                    } else {
                        response = vec!["OK", &ids, "false", &content];
                    }
                } else {
                    response = vec!["OK", &ids, "true", "duplicate"];
                }

                let response_str = vec![serde_json::to_string(&response)?];
                Ok(HandlerResult::Strings(response_str))
            }
            ClientMessage::Auth(auth) => {
                let ret = vec!["TODO".to_string()];
                Ok(HandlerResult::Strings(ret))
            }
            ClientMessage::Close(close) => {
                let ret = vec!["TODO".to_string()];
                Ok(HandlerResult::Strings(ret))
            }
            ClientMessage::NegClose { subscription_id } => {
                let ret = vec!["TODO".to_string()];
                Ok(HandlerResult::Strings(ret))
            }
            ClientMessage::Count {
                subscription_id,
                filters,
            } => {
                let ret = vec!["TODO".to_string()];
                Ok(HandlerResult::Strings(ret))
            }
            ClientMessage::Req { filters, .. } => {
                let order = Order::Desc;
                let queried_events = self.db.query(filters, order).await?;
                let mut ret = Vec::with_capacity(queried_events.len());
                for event in queried_events.into_iter() {
                    let response = serde_json::json!(["EVENT", "test", event]);
                    ret.push(response.to_string());
                }
                Ok(HandlerResult::Strings(ret))
            }
            ClientMessage::NegOpen {
                subscription_id,
                filter,
                id_size,
                initial_message,
            } => {
                let ret = vec!["TODO".to_string()];
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
