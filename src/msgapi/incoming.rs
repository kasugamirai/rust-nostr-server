use std::alloc::LayoutErr;
use std::iter::Successors;

use async_trait::async_trait;
use futures_util::future::ok;
use futures_util::sink::Close;
use futures_util::stream::Count;
use nostr::event::{kind, raw};
use nostr::{
    event, message::MessageHandleError, ClientMessage, Event, RawRelayMessage, RelayMessage,
};
use nostr::{JsonUtil, SubscriptionId};
use nostr_database::nostr;
use nostr_database::{DatabaseError, NostrDatabase, Order};
use nostr_rocksdb::RocksDatabase;
use tokio_tungstenite::tungstenite::http::response;

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

#[derive(Debug)]
pub struct Do<T> {
    data: T,
}

impl<T> Do<T> {
    pub async fn new(data: T) -> Self {
        Do { data }
    }
    pub async fn get_data(&self) -> &T {
        &self.data
    }
}

#[derive(Debug)]
pub enum HandlerResult {
    DoAuth(Do<String>),
    DoEvent(Do<String>),
    DoReq(Do<Vec<String>>),
    DoClose(Do<String>),
    DoCount(Do<String>),
    String(String),
    Strings(Vec<String>),
}

impl std::fmt::Display for HandlerResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::DoAuth(_) => write!(f, "DoAuth"),
            Self::DoEvent(_) => write!(f, "DoEvent"),
            Self::DoReq(_) => write!(f, "DoReq"),
            Self::DoClose(_) => write!(f, "DoClose"),
            Self::DoCount(_) => write!(f, "DoCount"),
            Self::String(_) => write!(f, "String"),
            Self::Strings(_) => write!(f, "Strings"),
        }
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
                let response: RelayMessage;
                let eid: nostr::EventId = event.id();
                let event_kind = event.kind();
                if event_kind == nostr::Kind::EventDeletion {
                    //TODO
                    let filter = nostr::Filter::new().event(eid);
                    self.db.delete(filter).await?;
                    let content: String = event.content().to_string();
                    response = RelayMessage::ok(eid, true, &content);
                    let response_str: String = serde_json::to_string(&response)?;
                    let ret = Do::new(response_str).await;
                    return Ok(HandlerResult::DoEvent(ret));
                }

                match self.check_signature(&event).await {
                    Ok(_) => {
                        // If the check_signature method was not successful
                        log::info!("Event signature is valid");
                    }
                    Err(e) => {
                        let err = e.to_string();
                        // If the check_signature method returned an error, handle it here
                        response = RelayMessage::ok(eid, false, &err);
                        let response_str = serde_json::to_string(&response)?;
                        let ret = Do::new(response_str).await;
                        return Ok(HandlerResult::DoEvent(ret));
                    }
                }

                let content: String = event.content().to_string();
                let event_existed: bool = self.db.has_event_already_been_saved(&eid).await?;
                if !event_existed && !event_kind.is_ephemeral() {
                    let success: bool = self.db.save_event(&event).await?;
                    if success {
                        response = RelayMessage::ok(eid, true, &content);
                    } else {
                        response = RelayMessage::ok(eid, false, &content);
                    }
                } else {
                    response = RelayMessage::ok(eid, true, DEDUPLICATED_EVENT);
                }
                let response_str: String = serde_json::to_string(&response)?;
                let ret = Do::new(response_str).await;
                Ok(HandlerResult::DoEvent(ret))
            }
            ClientMessage::Auth(auth) => {
                let event_id = auth.id();

                match self.check_signature(&auth).await {
                    Ok(_) => {
                        let status: bool = true;
                        let message: &str = "auth signature is valid";
                        let response: RelayMessage = RelayMessage::ok(event_id, status, message);
                        let response_str: String = serde_json::to_string(&response)?;
                        let ret = Do::new(response_str).await;
                        return Ok(HandlerResult::DoAuth(ret));
                    }
                    Err(e) => {
                        let err: String = e.to_string();
                        let status: bool = false;
                        let response: RelayMessage = RelayMessage::ok(event_id, status, &err);
                        let response_str: String = serde_json::to_string(&response)?;
                        let ret = Do::new(response_str).await;
                        return Ok(HandlerResult::DoAuth(ret));
                    }
                }
            }
            ClientMessage::Close(sid) => {
                let reason: String = String::from("received close message from client");
                let response: RelayMessage = RelayMessage::closed(sid, &reason);
                let ret = Do::new(serde_json::to_string(&response)?).await;
                Ok(HandlerResult::DoClose(ret))
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
                let count: usize = self.db.count(filters).await?;
                let response: RelayMessage = RelayMessage::count(subscription_id, count);
                let response_str: String = serde_json::to_string(&response)?;

                let ret: Do<String> = Do::new(response_str).await;
                Ok(HandlerResult::DoCount(ret))
            }
            ClientMessage::Req {
                subscription_id,
                filters,
            } => {
                let order: Order = Order::Desc;
                let queried_events: Vec<Event> = self.db.query(filters, order).await?;
                let mut ret: Vec<String> = Vec::with_capacity(queried_events.len());
                for e in queried_events.into_iter() {
                    let relay_messages: RelayMessage =
                        RelayMessage::event(subscription_id.clone(), e);
                    let serialized: String = serde_json::to_string(&relay_messages)?;
                    ret.push(serialized);
                }
                //let close_reason: &str = "reason";
                //let close_str: RelayMessage = RelayMessage::closed(subscription_id, close_reason);
                //let close_serialized: String = serde_json::to_string(&close_str)?;
                let end_of_send_event: RelayMessage = RelayMessage::eose(subscription_id);
                let end_of_send_event_str: String = serde_json::to_string(&end_of_send_event)?;
                ret.push(end_of_send_event_str);
                let ret = Do::new(ret).await;
                Ok(HandlerResult::DoReq(ret))
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
