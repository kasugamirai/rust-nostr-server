use std::alloc::LayoutErr;
use std::iter::Successors;

use async_trait::async_trait;
use futures_util::future::ok;
use futures_util::sink::Close;
use futures_util::stream::Count;
use nostr::event::raw;
use nostr::{
    event, message::MessageHandleError, ClientMessage, Event, RawRelayMessage, RelayMessage,
};
use nostr::{JsonUtil, SubscriptionId};
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

                match self.check_signature(*event.clone()).await {
                    Ok(_) => {
                        // If the check_signature method was not successful
                        log::info!("Event signature is valid");
                    }
                    Err(e) => {
                        let err = e.to_string();
                        // If the check_signature method returned an error, handle it here
                        response = RelayMessage::ok(eid, false, &err);
                        let response_str = serde_json::to_string(&response)?;
                        return Ok(HandlerResult::String(response_str));
                    }
                }
                let content = event.content().to_string();
                let event_existed = self.db.has_event_already_been_saved(&eid).await?;
                if !event_existed {
                    let success = self.db.save_event(&event).await?;
                    if success {
                        response = RelayMessage::ok(eid, true, &content);
                    } else {
                        response = RelayMessage::ok(eid, false, &content);
                    }
                } else {
                    response = RelayMessage::ok(eid, true, "deduplicated event");
                }
                let response_str = vec![serde_json::to_string(&response)?];
                Ok(HandlerResult::Strings(response_str))
            }
            ClientMessage::Auth(auth) => {
                let event_id = auth.id();

                match self.check_signature(*auth).await {
                    Ok(_) => {
                        let status: bool = true;
                        let message: &str = "auth signature is valid";
                        let response: RelayMessage = RelayMessage::ok(event_id, status, message);
                        let response_str: String = serde_json::to_string(&response)?;
                        return Ok(HandlerResult::String(response_str));
                    }
                    Err(e) => {
                        let err = e.to_string();
                        let status = false;
                        let response = RelayMessage::ok(event_id, status, &err);
                        let response_str = serde_json::to_string(&response)?;
                        return Ok(HandlerResult::String(response_str));
                    }
                }
            }
            ClientMessage::Close(sid) => {
                let reason = String::from("reason");
                let response = RelayMessage::closed(sid, &reason);
                let ret = serde_json::to_string(&response)?;
                Ok(HandlerResult::String(ret))
            }
            ClientMessage::NegClose { subscription_id } => {
                let ret = vec!["TODO".to_string()];
                Ok(HandlerResult::Strings(ret))
            }
            ClientMessage::Count {
                subscription_id,
                filters,
            } => {
                let count = self.db.count(filters).await?;
                let response = RelayMessage::count(subscription_id, count);
                let response_str = serde_json::to_string(&response)?;
                Ok(HandlerResult::String(response_str))
            }
            ClientMessage::Req {
                subscription_id,
                filters,
            } => {
                let order = Order::Desc;

                let queried_events = self.db.query(filters, order).await?;
                let mut ret = Vec::with_capacity(queried_events.len());
                for e in queried_events.into_iter() {
                    let relay_messages = RelayMessage::event(subscription_id.clone(), e);
                    let serialized = serde_json::to_string(&relay_messages)?;
                    ret.push(serialized);
                }
                let close_reason = "reason";
                let close_str = RelayMessage::closed(subscription_id, close_reason);
                let close_serialized = serde_json::to_string(&close_str)?;
                ret.push(close_serialized);
                Ok(HandlerResult::Strings(ret))
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
