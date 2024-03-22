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
pub struct DoClose {
    reason: String,
}

impl DoClose {
    pub async fn new(reason: &str) -> Self {
        DoClose {
            reason: reason.to_string(),
        }
    }
    pub async fn get_reason(&self) -> String {
        self.reason.clone()
    }
}

#[derive(Debug)]
pub struct DoEvent {
    event: String,
}

impl DoEvent {
    pub async fn new(event: String) -> Self {
        DoEvent { event }
    }
    pub async fn get_event(&self) -> String {
        self.event.clone()
    }
}

#[derive(Debug)]
pub struct DoReq {
    req: Vec<String>,
}

impl DoReq {
    pub async fn new(req: Vec<String>) -> Self {
        DoReq { req }
    }
    pub async fn get_req(&self) -> Vec<String> {
        self.req.clone()
    }
}

#[derive(Debug)]
pub struct DoAuth {
    auth: String,
}

impl DoAuth {
    pub async fn new(auth: String) -> Self {
        DoAuth { auth }
    }
    pub async fn get_auth(&self) -> String {
        self.auth.clone()
    }
}

#[derive(Debug)]
pub struct DoCount {
    count: String,
}

impl DoCount {
    pub async fn new(count: String) -> Self {
        DoCount { count }
    }
    pub async fn get_count(&self) -> String {
        self.count.clone()
    }
}

#[derive(Debug)]
pub struct DoNegClose {
    neg_close: String,
}

impl DoNegClose {
    pub async fn new(neg_close: String) -> Self {
        DoNegClose { neg_close }
    }
    pub async fn get_neg_close(&self) -> String {
        self.neg_close.clone()
    }
}

#[derive(Debug)]
pub struct DoNegMsg {
    neg_msg: String,
}

impl DoNegMsg {
    pub async fn new(neg_msg: String) -> Self {
        DoNegMsg { neg_msg }
    }
    pub async fn get_neg_msg(&self) -> String {
        self.neg_msg.clone()
    }
}

#[derive(Debug)]
pub struct DoNegOpen {
    neg_open: String,
}

impl DoNegOpen {
    pub async fn new(neg_open: String) -> Self {
        DoNegOpen { neg_open }
    }
    pub async fn get_neg_open(&self) -> String {
        self.neg_open.clone()
    }
}

#[derive(Debug)]
pub enum HandlerResult {
    DoAuth(DoAuth),
    DoEvent(DoEvent),
    DoReq(DoReq),
    DoClose(DoClose),
    DoCount(DoCount),
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
    async fn check_signature(&self, event: Event) -> Result<(), Error>;
}

#[async_trait]
impl MessageHandler for IncomingMessage {
    async fn check_signature(&self, event: Event) -> Result<(), Error> {
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
                        let ret = DoEvent::new(response_str).await;
                        return Ok(HandlerResult::DoEvent(ret));
                    }
                }
                let content: String = event.content().to_string();
                let event_existed: bool = self.db.has_event_already_been_saved(&eid).await?;
                if !event_existed {
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
                let ret: DoEvent = DoEvent::new(response_str).await;
                Ok(HandlerResult::DoEvent(ret))
            }
            ClientMessage::Auth(auth) => {
                let event_id = auth.id();

                match self.check_signature(*auth).await {
                    Ok(_) => {
                        let status: bool = true;
                        let message: &str = "auth signature is valid";
                        let response: RelayMessage = RelayMessage::ok(event_id, status, message);
                        let response_str: String = serde_json::to_string(&response)?;
                        let ret: DoAuth = DoAuth::new(response_str).await;
                        return Ok(HandlerResult::DoAuth(ret));
                    }
                    Err(e) => {
                        let err: String = e.to_string();
                        let status: bool = false;
                        let response: RelayMessage = RelayMessage::ok(event_id, status, &err);
                        let response_str: String = serde_json::to_string(&response)?;
                        let ret: DoAuth = DoAuth::new(response_str).await;
                        return Ok(HandlerResult::DoAuth(ret));
                    }
                }
            }
            ClientMessage::Close(sid) => {
                let reason: String = String::from("received close message from client");
                let response: RelayMessage = RelayMessage::closed(sid, &reason);
                let ret: DoClose = DoClose::new(&serde_json::to_string(&response)?).await;
                Ok(HandlerResult::DoClose(ret))
            }
            ClientMessage::NegClose { subscription_id } => {
                //TODO: Implement this
                let CloseMessage = "close message";
                let response: RelayMessage = RelayMessage::closed(subscription_id, CloseMessage);
                let ret: DoNegClose;
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

                let ret: DoCount = DoCount::new(response_str).await;
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
                let close_reason: &str = "reason";
                let close_str: RelayMessage = RelayMessage::closed(subscription_id, close_reason);
                let close_serialized: String = serde_json::to_string(&close_str)?;
                ret.push(close_serialized);
                let ret: DoReq = DoReq::new(ret).await;
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
