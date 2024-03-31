use nostr::SubscriptionId;
use nostr::{event, message::MessageHandleError, ClientMessage, Event, RelayMessage};
use nostr_database::{NostrDatabase, Order};
use nostr_rocksdb::RocksDatabase;

const DEDUPLICATED_EVENT: &'static str = "deduplicated event";
const EVENT_SIGNATURE_VALID: &'static str = "event signature is valid";
const close_message: &'static str = "received close message from client";
const database_path: &'static str = "./db/rocksdb";

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
        let db = RocksDatabase::open(database_path).await?;
        Ok(Self { db })
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
    Auth(OperationData<String>, bool),
    Event(OperationData<String>),
    Req(OperationData<Vec<String>>),
    Close(OperationData<String>),
    Count(OperationData<String>),
    String(String),
    Strings(Vec<String>),
}

impl std::fmt::Display for HandlerResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Auth(_, _) => write!(f, "Auth"),
            Self::Event(_) => write!(f, "Event"),
            Self::Req(_) => write!(f, "Req"),
            Self::Close(_) => write!(f, "Close"),
            Self::Count(_) => write!(f, "Count"),
            Self::String(_) => write!(f, "String"),
            Self::Strings(_) => write!(f, "Strings"),
        }
    }
}

impl IncomingMessage {
    fn check_signature<'a>(&self, event: &'a Event) -> Result<(), Error> {
        let ret = event.verify_signature()?;
        Ok(ret)
    }
    pub async fn to_client_message(&self, txt: &str) -> Result<ClientMessage, Error> {
        let ret: ClientMessage = <ClientMessage as nostr::JsonUtil>::from_json(txt)?;
        Ok(ret)
    }
    pub async fn handlers(
        &self,
        client_message: ClientMessage,
        certified: bool,
    ) -> Result<HandlerResult, Error> {
        match client_message {
            ClientMessage::Event(event) => {
                if certified {
                    //todo: handle event
                }
                let ret = self.handle_event(event).await?;
                Ok(ret)
            }
            ClientMessage::Auth(auth) => {
                let ret = self.handle_auth(auth).await?;
                Ok(ret)
            }
            ClientMessage::Close(sid) => {
                let ret = self.handle_close(sid).await?;
                Ok(ret)
            }
            ClientMessage::NegClose { subscription_id } => {
                let ret = self.handle_neg_close(subscription_id).await?;
                Ok(ret)
            }
            ClientMessage::Count {
                subscription_id,
                filters,
            } => {
                let ret = self.handle_count(subscription_id, filters).await?;
                Ok(ret)
            }
            ClientMessage::Req {
                subscription_id,
                filters,
            } => {
                let ret = self.handle_req(subscription_id, filters).await?;
                Ok(ret)
            }
            ClientMessage::NegOpen {
                subscription_id,
                filter,
                id_size,
                initial_message,
            } => {
                let ret = self
                    .handle_neg_open(subscription_id, filter, id_size, initial_message)
                    .await?;
                Ok(ret)
            }
            ClientMessage::NegMsg {
                subscription_id,
                message,
            } => {
                let ret = self.handle_neg_msg(subscription_id, message).await?;
                Ok(ret)
            }
        }
    }
}

impl IncomingMessage {
    async fn handle_auth(&self, auth: Box<Event>) -> Result<HandlerResult, Error> {
        let event_id = auth.id();

        match self.check_signature(&auth) {
            Ok(_) => {
                let status: bool = true;
                let response: RelayMessage =
                    RelayMessage::ok(event_id, status, EVENT_SIGNATURE_VALID);
                let response_str: String = serde_json::to_string(&response)?;
                let ret = OperationData::new(response_str).await;
                return Ok(HandlerResult::Auth(ret, status));
            }
            Err(e) => {
                let err: String = e.to_string();
                let status: bool = false;
                let response: RelayMessage = RelayMessage::ok(event_id, status, &err);
                let response_str: String = serde_json::to_string(&response)?;
                let ret = OperationData::new(response_str).await;
                return Ok(HandlerResult::Auth(ret, status));
            }
        }
    }

    async fn handle_close(&self, sid: SubscriptionId) -> Result<HandlerResult, Error> {
        let response: RelayMessage = RelayMessage::closed(sid, close_message);
        let ret = OperationData::new(serde_json::to_string(&response)?).await;
        Ok(HandlerResult::Close(ret))
    }

    async fn handle_neg_close(&self, sid: SubscriptionId) -> Result<HandlerResult, Error> {
        let CloseMessage = "close message";
        let response: RelayMessage = RelayMessage::closed(sid, CloseMessage);
        let ret: Vec<String> = vec!["TODO".to_string()];
        Ok(HandlerResult::Strings(ret))
    }

    async fn handle_count(
        &self,
        sid: SubscriptionId,
        filters: Vec<nostr::Filter>,
    ) -> Result<HandlerResult, Error> {
        let count: usize = self.db.count(filters).await?;
        let response: RelayMessage = RelayMessage::count(sid, count);
        let response_str: String = serde_json::to_string(&response)?;

        let ret: OperationData<String> = OperationData::new(response_str).await;
        Ok(HandlerResult::Count(ret))
    }

    async fn handle_req(
        &self,
        sid: SubscriptionId,
        filters: Vec<nostr::Filter>,
    ) -> Result<HandlerResult, Error> {
        let order: Order = Order::Desc;
        let queried_events: Vec<Event> = self.db.query(filters, order).await?;
        let mut ret: Vec<String> = Vec::with_capacity(queried_events.len());
        for e in queried_events.into_iter() {
            let relay_messages: RelayMessage = RelayMessage::event(sid.clone(), e);
            let serialized: String = serde_json::to_string(&relay_messages)?;
            ret.push(serialized);
        }
        let end_of_send_event: RelayMessage = RelayMessage::eose(sid);
        let end_of_send_event_str: String = serde_json::to_string(&end_of_send_event)?;
        ret.push(end_of_send_event_str);
        let ret = OperationData::new(ret).await;
        Ok(HandlerResult::Req(ret))
    }

    async fn handle_neg_open(
        &self,
        sid: SubscriptionId,
        filter: Box<nostr::Filter>,
        id_size: u8,
        initial_message: String,
    ) -> Result<HandlerResult, Error> {
        let ret: Vec<String> = vec!["TODO".to_string()];
        Ok(HandlerResult::Strings(ret))
    }

    async fn handle_neg_msg(
        &self,
        sid: SubscriptionId,
        message: String,
    ) -> Result<HandlerResult, Error> {
        let ret = vec!["TODO".to_string()];
        Ok(HandlerResult::Strings(ret))
    }

    async fn handle_event(&self, event: Box<Event>) -> Result<HandlerResult, Error> {
        let response: RelayMessage;
        let eid: nostr::EventId = event.id();
        let event_kind = event.kind();
        if event_kind == nostr::Kind::EventDeletion {
            let filter = nostr::Filter::new().event(eid);
            self.db.delete(filter).await?;
            let content: String = event.content().to_string();
            response = RelayMessage::ok(eid, true, &content);
            let response_str: String = serde_json::to_string(&response)?;
            let ret = OperationData::new(response_str).await;
            return Ok(HandlerResult::Event(ret));
        }

        match self.check_signature(&event) {
            Ok(_) => {
                log::info!("Event signature is valid");
            }
            Err(e) => {
                let err = e.to_string();
                response = RelayMessage::ok(eid, false, &err);
                let response_str = serde_json::to_string(&response)?;
                let ret = OperationData::new(response_str).await;
                return Ok(HandlerResult::Event(ret));
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
        let ret = OperationData::new(response_str).await;
        Ok(HandlerResult::Event(ret))
    }
}
