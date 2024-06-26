use async_trait::async_trait;
//use nostr::message::subscription;
use nostr::RelayMessage;
use nostr::SubscriptionId;
use std::default::Default;
const UNAUTHENTICATED: &str = "we can't serve DMs to unauthenticated users";
use super::HandlerResult;
use super::OperationData;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("to client message: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

#[derive(Default)]
pub struct OutgoingMessage {}

#[async_trait]
pub trait OutgoingHandler {
    async fn send_challenge<'a, 'b: 'a>(
        &'a self,
        challenge_msg: &'b str,
    ) -> Result<HandlerResult, Error>;
    async fn send_notice(&self, notice_msg: String) -> Result<HandlerResult, Error>;
    async fn send_eose(&self, subscription_id: SubscriptionId) -> Result<String, Error>;
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
    async fn send_eose(&self, subscription_id: SubscriptionId) -> Result<String, Error> {
        let end_of_send_event: RelayMessage = RelayMessage::eose(subscription_id);
        let end_of_send_event_str: String = serde_json::to_string(&end_of_send_event)?;
        Ok(end_of_send_event_str)
    }
}
