use async_trait::async_trait;
use nostr::message::subscription;
use nostr::RelayMessage;
use nostr::{event, SubscriptionId};

#[derive(Debug)]
pub enum Error {
    Event(event::Error),
    ToClientMessage(serde_json::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Event(e) => write!(f, "event: {}", e),
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

pub struct OutgoingMessage {}

impl OutgoingMessage {
    pub async fn new() -> Self {
        OutgoingMessage {}
    }
}

#[derive(Debug)]
pub struct challenge_msg {
    pub challenge_msg: String,
}
#[derive(Debug)]
pub struct notice_msg {
    pub notice_msg: String,
}

impl challenge_msg {
    pub async fn new(challenge_msg: String) -> Self {
        challenge_msg { challenge_msg }
    }
    pub async fn get_challenge_msg(&self) -> String {
        self.challenge_msg.clone()
    }
}

impl notice_msg {
    pub async fn new(notice_msg: String) -> Self {
        notice_msg { notice_msg }
    }
    pub async fn get_notice_msg(&self) -> String {
        self.notice_msg.clone()
    }
}

#[derive(Debug, Clone)]
pub struct EoseMsg {
    pub eose_msg: String,
}

impl EoseMsg {
    pub async fn new(eose_msg: String) -> Self {
        EoseMsg { eose_msg }
    }
    pub async fn get_subscription_id(&self) -> String {
        self.eose_msg.clone()
    }
}

#[derive(Debug)]
pub enum OutgoingMessageTypes {
    Challenge(challenge_msg),
    Notice(notice_msg),
    Eose(EoseMsg),
}
#[async_trait]
pub trait OutgoingHandler {
    async fn send_challenge(&self, challenge_msg: String) -> Result<OutgoingMessageTypes, Error>;
    async fn send_notice(&self, notice_msg: String) -> Result<OutgoingMessageTypes, Error>;
    async fn send_eose(
        &self,
        subscription_id: SubscriptionId,
    ) -> Result<OutgoingMessageTypes, Error>;
}

#[async_trait]
impl OutgoingHandler for OutgoingMessage {
    async fn send_challenge(&self, challenge_msg: String) -> Result<OutgoingMessageTypes, Error> {
        let relay_message: RelayMessage = RelayMessage::auth(challenge_msg);
        let challenge_str: String = serde_json::to_string(&relay_message)?;
        let ret = challenge_msg::new(challenge_str).await;
        Ok(OutgoingMessageTypes::Challenge(ret))
    }
    async fn send_notice(&self, notice_msg: String) -> Result<OutgoingMessageTypes, Error> {
        let relay_message: RelayMessage = RelayMessage::notice(notice_msg);
        let notice_str: String = serde_json::to_string(&relay_message)?;
        let ret = notice_msg::new(notice_str).await;
        Ok(OutgoingMessageTypes::Notice(ret))
    }
    async fn send_eose(
        &self,
        subscription_id: SubscriptionId,
    ) -> Result<OutgoingMessageTypes, Error> {
        let end_of_send_event: RelayMessage = RelayMessage::eose(subscription_id);
        let end_of_send_event_str: String = serde_json::to_string(&end_of_send_event)?;
        let ret = EoseMsg::new(end_of_send_event_str).await;
        Ok(OutgoingMessageTypes::Eose(ret))
    }
}
