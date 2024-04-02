mod incoming;
mod outgoing;
pub use incoming::IncomingMessage;
pub use outgoing::OutgoingHandler;
pub use outgoing::OutgoingMessage;
#[derive(Debug)]
pub enum Error {
    Incoming(incoming::Error),
    Outgoing(outgoing::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Incoming(e) => write!(f, "incoming: {}", e),
            Self::Outgoing(e) => write!(f, "outgoing: {}", e),
        }
    }
}

mod challange;

pub use challange::Challenge;
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
