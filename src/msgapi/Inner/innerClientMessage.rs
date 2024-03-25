use futures_util::future::ok;
use nostr::event::Event;
use nostr::message;
use nostr::ClientMessage;
use nostr::Filter;
use nostr::RelayMessage;
use nostr::SubscriptionId;
use nostr_database::{DatabaseError, NostrDatabase, Order};
use nostr_rocksdb::RocksDatabase;

const DEDUPLICATED_EVENT: &str = "deduplicated event";
const EVENT_SIGNATURE_VALID: &str = "event signature is valid";

pub struct InnerEvent {
    event: Box<Event>,
    db: RocksDatabase,
}

impl InnerEvent {
    pub async fn new(event: Box<Event>, db: RocksDatabase) -> Self {
        InnerEvent { event, db }
    }
}

pub struct InnerReq {
    subscription: SubscriptionId,
    filter: Vec<Filter>,
    db: RocksDatabase,
}

impl InnerReq {
    pub async fn new(subscription: SubscriptionId, filter: Vec<Filter>, db: RocksDatabase) -> Self {
        InnerReq {
            subscription,
            filter,
            db,
        }
    }
}

pub struct InnerCount {
    subscription: SubscriptionId,
    filter: Vec<Filter>,
    db: RocksDatabase,
}

impl InnerCount {
    pub async fn new(subscription: SubscriptionId, filter: Vec<Filter>, db: RocksDatabase) -> Self {
        InnerCount {
            subscription,
            filter,
            db,
        }
    }
}

pub struct InnerClose {
    subscription: SubscriptionId,
}

impl InnerClose {
    pub async fn new(subscription: SubscriptionId) -> Self {
        InnerClose { subscription }
    }
}

pub struct InnerAuth {
    event: Box<Event>,
}

impl InnerAuth {
    pub async fn new(event: Box<Event>) -> Self {
        InnerAuth { event }
    }
}

pub struct InnerNegOpen {
    subscription: SubscriptionId,
    filter: Box<Filter>,
    id_size: u8,
    initial_message: String,
}

pub struct InnerNegClose {
    subscription: SubscriptionId,
}

pub struct InnerNegMsg {
    subscription: SubscriptionId,
    msg: String,
}

#[derive(Debug)]
pub enum Error {
    Event(nostr::event::Error),
    Database(DatabaseError),
    Serialize(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Event(e) => write!(f, "event: {}", e),
            Self::Database(e) => write!(f, "database: {}", e),
            Self::Serialize(e) => write!(f, "serialize: {}", e),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Serialize(e)
    }
}

impl From<DatabaseError> for Error {
    fn from(e: DatabaseError) -> Self {
        Self::Database(e)
    }
}

impl From<nostr::event::Error> for Error {
    fn from(e: nostr::event::Error) -> Self {
        Self::Event(e)
    }
}

#[derive(Debug)]
pub struct OperationData<Data> {
    data: Data,
}
impl<Data> OperationData<Data> {
    pub async fn new(data: Data) -> Self {
        OperationData { data }
    }
    pub async fn get_data(&self) -> &Data {
        &self.data
    }
}

#[derive(Debug)]
pub enum HandlerResult {
    DoAuth(OperationData<String>),
    DoEvent(OperationData<String>),
    DoReq(OperationData<Vec<String>>),
    DoClose(OperationData<String>),
    DoCount(OperationData<String>),
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

pub trait MessageHandle {
    async fn handle(&self) -> Result<HandlerResult, Error>;
}

impl MessageHandle for InnerEvent {
    async fn handle(&self) -> Result<HandlerResult, Error> {
        let response: RelayMessage;
        let event_id = self.event.id;
        let event_kind = self.event.kind;
        if event_kind == nostr::Kind::EventDeletion {
            let filter = nostr::Filter::new().event(event_id);
            self.db.delete(filter).await?;
            let content: String = self.event.content().to_string();
            response = RelayMessage::ok(event_id, true, &content);
            let response_str: String = serde_json::to_string(&response)?;
            let ret: OperationData<String> = OperationData::new(response_str).await;
            return Ok(HandlerResult::DoEvent(ret));
        }

        self.event.verify_signature()?;

        let content: String = self.event.content().to_string();
        let event_existed: bool = self.db.has_event_already_been_saved(&event_id).await?;
        if !event_existed && !event_kind.is_ephemeral() {
            let success: bool = self.db.save_event(&self.event).await?;
            if success {
                response = RelayMessage::ok(event_id, true, &content);
            } else {
                response = RelayMessage::ok(event_id, false, &content);
            }
        } else {
            response = RelayMessage::ok(event_id, true, DEDUPLICATED_EVENT);
        }
        let response_str: String = serde_json::to_string(&response)?;
        let ret = OperationData::new(response_str).await;
        Ok(HandlerResult::DoEvent(ret))
    }
}

impl MessageHandle for InnerReq {
    async fn handle(&self) -> Result<HandlerResult, Error> {
        let order: Order = Order::Desc;
        let queried_events: Vec<Event> = self.db.query(self.filter.clone(), order).await?;
        let mut ret: Vec<String> = Vec::with_capacity(queried_events.len());
        for e in queried_events.into_iter() {
            let relay_messages: RelayMessage = RelayMessage::event(self.subscription.clone(), e);
            let serialized: String = serde_json::to_string(&relay_messages)?;
            ret.push(serialized);
        }
        //let close_reason: &str = "reason";
        //let close_str: RelayMessage = RelayMessage::closed(subscription_id, close_reason);
        //let close_serialized: String = serde_json::to_string(&close_str)?;
        let end_of_send_event: RelayMessage = RelayMessage::eose(self.subscription.clone());
        let end_of_send_event_str: String = serde_json::to_string(&end_of_send_event)?;
        ret.push(end_of_send_event_str);
        let ret = OperationData::new(ret).await;
        Ok(HandlerResult::DoReq(ret))
    }
}

impl MessageHandle for InnerCount {
    async fn handle(&self) -> Result<HandlerResult, Error> {
        let count: usize = self.db.count(self.filter.clone()).await?;
        let response: RelayMessage = RelayMessage::count(self.subscription.clone(), count);
        let response_str: String = serde_json::to_string(&response)?;
        let ret: OperationData<String> = OperationData::new(response_str).await;
        Ok(HandlerResult::DoCount(ret))
    }
}

impl MessageHandle for InnerAuth {
    async fn handle(&self) -> Result<HandlerResult, Error> {
        let event_id = self.event.id();
        match self.event.verify_signature() {
            Ok(_) => {
                let status: bool = true;
                let message: &str = "auth signature is valid";
                let response: RelayMessage = RelayMessage::ok(event_id, status, message);
                let response_str: String = serde_json::to_string(&response)?;
                let ret = OperationData::new(response_str).await;
                return Ok(HandlerResult::DoAuth(ret));
            }
            Err(e) => {
                let err: String = e.to_string();
                let status: bool = false;
                let response: RelayMessage = RelayMessage::ok(event_id, status, &err);
                let response_str: String = serde_json::to_string(&response)?;
                let ret: OperationData<String> = OperationData::new(response_str).await;
                return Ok(HandlerResult::DoAuth(ret));
            }
        }
    }
}

impl MessageHandle for InnerClose {
    async fn handle(&self) -> Result<HandlerResult, Error> {
        let subscription_id: SubscriptionId = self.subscription.clone();
        let close_reason: &str = "reason";
        let close_str: RelayMessage = RelayMessage::closed(subscription_id, close_reason);
        let close_serialized: String = serde_json::to_string(&close_str)?;
        let ret: OperationData<String> = OperationData::new(close_serialized).await;
        Ok(HandlerResult::DoClose(ret))
    }
}

impl MessageHandle for InnerNegOpen {
    async fn handle(&self) -> Result<HandlerResult, Error> {
        Ok(HandlerResult::String("TODO".to_string()))
    }
}

impl MessageHandle for InnerNegClose {
    async fn handle(&self) -> Result<HandlerResult, Error> {
        Ok(HandlerResult::String("TODO".to_string()))
    }
}

impl MessageHandle for InnerNegMsg {
    async fn handle(&self) -> Result<HandlerResult, Error> {
        Ok(HandlerResult::String("TODO".to_string()))
    }
}
