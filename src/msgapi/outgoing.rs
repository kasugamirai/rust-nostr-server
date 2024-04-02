use async_trait::async_trait;
use nostr::message::subscription;
use nostr::RelayMessage;
use nostr::{event, SubscriptionId};

const unauthenticated: &'static str = "we can't serve DMs to unauthenticated users";
use super::Error;
use super::HandlerResult;
use super::OperationData;

pub struct OutgoingMessage {}

impl OutgoingMessage {
    pub fn new() -> Self {
        OutgoingMessage {}
    }
}

#[async_trait]
pub trait OutgoingHandler {
    async fn send_challenge<'a, 'b: 'a>(
        &'a self,
        challenge_msg: &'b str,
    ) -> Result<HandlerResult, Error>;
    async fn send_notice(&self, notice_msg: String) -> Result<HandlerResult, Error>;
    async fn send_eose(&self, subscription_id: SubscriptionId) -> Result<HandlerResult, Error>;
}

#[async_trait]
impl OutgoingHandler for OutgoingMessage {
    async fn send_challenge<'a, 'b: 'a>(
        &'a self,
        challenge_msg: &'b str,
    ) -> Result<HandlerResult, Error> {
        let relay_message: RelayMessage = RelayMessage::auth(challenge_msg);
        let challenge_str: String = serde_json::to_string(&relay_message)?;
        let ret = OperationData::new(challenge_str);
        Ok(HandlerResult::Challenge(ret))
    }
    async fn send_notice(&self, notice_msg: String) -> Result<HandlerResult, Error> {
        let relay_message: RelayMessage = RelayMessage::notice(notice_msg);
        let notice_str: String = serde_json::to_string(&relay_message)?;
        let ret = OperationData::new(notice_str);
        Ok(HandlerResult::Notice(ret))
    }
    async fn send_eose(&self, subscription_id: SubscriptionId) -> Result<HandlerResult, Error> {
        let end_of_send_event: RelayMessage = RelayMessage::eose(subscription_id);
        let end_of_send_event_str: String = serde_json::to_string(&end_of_send_event)?;
        let ret = OperationData::new(end_of_send_event_str);
        Ok(HandlerResult::Eose(ret))
    }
}
