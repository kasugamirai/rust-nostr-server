mod incoming;
mod outgoing;
pub use incoming::IncomingMessage;
pub use outgoing::OutgoingMessage;

mod challange;
pub use outgoing::OutgoingHandler;

pub use challange::Challenge;
#[derive(Debug)]
pub struct OperationData<Data> {
    data: Data,
}
impl<Data> OperationData<Data> {
    pub fn new(data: Data) -> Self {
        OperationData { data }
    }
    pub fn get_data(&self) -> &Data {
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
    Challenge(OperationData<String>),
    Notice(OperationData<String>),
    Eose(OperationData<String>),
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
            Self::Challenge(_) => write!(f, "Challenge"),
            Self::Notice(_) => write!(f, "Notice"),
            Self::Eose(_) => write!(f, "Eose"),
            Self::String(_) => write!(f, "String"),
            Self::Strings(_) => write!(f, "Strings"),
        }
    }
}

/*
#[derive(Debug)]
pub enum Error {
    Event(event::Error),
    MessageHandle(nostr::message::MessageHandleError),
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

impl From<nostr::message::MessageHandleError> for Error {
    fn from(e: nostr::message::MessageHandleError) -> Self {
        Self::MessageHandle(e)
    }
}
*/
