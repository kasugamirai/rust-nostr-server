use async_trait::async_trait;
use futures_util::future::ok;
use nostr::JsonUtil;
use nostr::{
    event, message::MessageHandleError, ClientMessage, Event, RawRelayMessage, RelayMessage,
};
use nostr_database::nostr;
use nostr_database::{DatabaseError, NostrDatabase, Order};
use nostr_rocksdb::RocksDatabase;

#[derive(Debug, Clone)]
pub struct IncomingMessage {
    db: RocksDatabase,
}

#[derive(Debug)]
pub enum Error {
    Event(event::Error),
    MessageHandleError(MessageHandleError),
    DatabaseError(nostr_rocksdb::database::DatabaseError),
    ToClientMessage(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Event(e) => write!(f, "event: {}", e),
            Self::MessageHandleError(e) => write!(f, "message handle error: {}", e),
            Self::DatabaseError(e) => write!(f, "database error: {}", e),
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
        Self::DatabaseError(e)
    }
}

impl From<MessageHandleError> for Error {
    fn from(e: MessageHandleError) -> Self {
        Self::MessageHandleError(e)
    }
}

impl IncomingMessage {
    pub async fn new() -> Result<Self, Error> {
        let db = RocksDatabase::open("./db/rocksdb").await?;
        Ok(IncomingMessage { db })
    }
}

#[async_trait]
pub trait MessageHandler {
    async fn handlers(&self, ClientMessage: ClientMessage) -> Result<Vec<String>, Error>;
    async fn to_client_message(&self, txt: &str) -> Result<ClientMessage, Error>;
}

#[async_trait]
impl MessageHandler for IncomingMessage {
    async fn to_client_message(&self, txt: &str) -> Result<ClientMessage, Error> {
        let ret = <ClientMessage as nostr::JsonUtil>::from_json(txt)?;
        Ok(ret)
    }
async fn handlers(&self, ClientMessage: ClientMessage) -> Result<Vec<String>, Error> {
        match ClientMessage {
            ClientMessage::Event(event) => {
                let eid = event.id();
                let event_existed = self.db.has_event_already_been_saved(&eid).await?;
                self.db.save_event(&event).await?;
                //let raw_client_message = Message::Text(Event::as_json(&event));
                //let messages = vec![&raw_client_message];
                let ids = event.id().to_string();
                let context = event.content().to_string();
                let response = vec!["OK", &ids, "true", &context];

                let response_str = vec![serde_json::to_string(&response)?];
                Ok(response_str)
            }
            ClientMessage::Auth(auth) => {
                let ret = vec!["TODO".to_string()];
                Ok(ret)
            }
            ClientMessage::Close(close) => {
                let ret = vec!["TODO".to_string()];
                Ok(ret)
            }
            ClientMessage::NegClose { subscription_id } => {
                let ret = vec!["TODO".to_string()];
                Ok(ret)
            }
            ClientMessage::Count {
                subscription_id,
                filters,
            } => {
                let ret = vec!["TODO".to_string()];
                Ok(ret)
            }
            ClientMessage::Req {
                subscription_id,
                filters,
            } => {
                let order = Order::Desc;
                let queried_events = self.db.query(filters, order).await?;
                let mut ret = vec![];
                for event in &queried_events {
                    let raw = Event::as_json(&event);
                    let event_json: serde_json::Value = serde_json::from_str(&raw)?;
                    let response = serde_json::json!(["EVENT", "test", event_json]);
                    ret.push(response.to_string());
                }
                Ok(ret)
            }
            ClientMessage::NegOpen {
                subscription_id,
                filter,
                id_size,
                initial_message,
            } => {
                let ret = vec!["TODO".to_string()];
                Ok(ret)
            }
            ClientMessage::NegMsg {
                subscription_id,
                message,
            } => {
                let ret = vec!["TODO".to_string()];
                Ok(ret)
            }
        }
    }
}
