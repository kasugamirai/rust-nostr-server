use nostr::SubscriptionId;
use nostr::{ClientMessage, Event, RelayMessage};
use nostr_database::{NostrDatabase, Order};
use nostr_rocksdb::RocksDatabase;

const DEDUPLICATED_EVENT: &'static str = "deduplicated event";
const EVENT_SIGNATURE_VALID: &'static str = "event signature is valid";
const CLOSE_MESSAGE: &'static str = "received close message from client";
const DATABASE_PATH: &'static str = "./db/rocksdb";
use super::Error;
use super::OperationData;
use super::OutgoingHandler;
use crate::HandlerResult;

use super::Challenge;
use super::OutgoingMessage;

const CHALLENGE_MESSAGE: &'static str = "helloWorld";
#[derive(Debug, Clone)]
pub struct IncomingMessage {
    db: RocksDatabase,
}

impl IncomingMessage {
    pub async fn new() -> Result<Self, Error> {
        let db = RocksDatabase::open(DATABASE_PATH).await?;
        Ok(Self { db })
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
                if !certified {
                    if is_channel_message(&event) {}
                    //todo: handle event
                }
                let ret = self.handle_event(event, certified).await?;
                Ok(ret)
            }
            ClientMessage::Auth(auth) => {
                let ret = self.handle_auth(auth).await?;
                //todo: send challenge message
                //todo: set certified to true
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
        let challenge = Challenge::new(&auth);
        let time_check = challenge.time_check();
        let signature_check = challenge.signature_check();
        let client_authentication = challenge.client_authentication();
        let relay_check = challenge.relay_check();
        if !time_check {
            let response: RelayMessage = RelayMessage::ok(event_id, false, "time check failed");
            let response_str: String = serde_json::to_string(&response)?;
            let ret = OperationData::new(response_str);
            return Ok(HandlerResult::Auth(ret, false));
        }
        if !signature_check {
            let response: RelayMessage =
                RelayMessage::ok(event_id, false, "signature check failed");
            let response_str: String = serde_json::to_string(&response)?;
            let ret = OperationData::new(response_str);
            return Ok(HandlerResult::Auth(ret, false));
        }
        if !client_authentication {
            let response: RelayMessage =
                RelayMessage::ok(event_id, false, "client authentication failed");
            let response_str: String = serde_json::to_string(&response)?;
            let ret = OperationData::new(response_str);
            return Ok(HandlerResult::Auth(ret, false));
        }
        if !relay_check {
            let response: RelayMessage = RelayMessage::ok(event_id, false, "relay check failed");
            let response_str: String = serde_json::to_string(&response)?;
            let ret = OperationData::new(response_str);
            return Ok(HandlerResult::Auth(ret, false));
        }

        let status: bool = true;
        let response: RelayMessage = RelayMessage::ok(event_id, status, EVENT_SIGNATURE_VALID);
        let response_str: String = serde_json::to_string(&response)?;
        let ret = OperationData::new(response_str);
        return Ok(HandlerResult::Auth(ret, status));
    }

    async fn handle_close(&self, sid: SubscriptionId) -> Result<HandlerResult, Error> {
        let response: RelayMessage = RelayMessage::closed(sid, CLOSE_MESSAGE);
        let ret = OperationData::new(serde_json::to_string(&response)?);
        Ok(HandlerResult::Close(ret))
    }

    async fn handle_neg_close(&self, sid: SubscriptionId) -> Result<HandlerResult, Error> {
        let close_message = "close message";
        let response: RelayMessage = RelayMessage::closed(sid, close_message);
        todo!("handle_neg_close")
    }

    async fn handle_count(
        &self,
        sid: SubscriptionId,
        filters: Vec<nostr::Filter>,
    ) -> Result<HandlerResult, Error> {
        let count: usize = self.db.count(filters).await?;
        let response: RelayMessage = RelayMessage::count(sid, count);
        let response_str: String = serde_json::to_string(&response)?;

        let ret: OperationData<String> = OperationData::new(response_str);
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
        let ret = OperationData::new(ret);
        Ok(HandlerResult::Req(ret))
    }

    async fn handle_neg_open(
        &self,
        sid: SubscriptionId,
        filter: Box<nostr::Filter>,
        id_size: u8,
        initial_message: String,
    ) -> Result<HandlerResult, Error> {
        todo!("handle_neg_open");
    }

    async fn handle_neg_msg(
        &self,
        sid: SubscriptionId,
        message: String,
    ) -> Result<HandlerResult, Error> {
        todo!("handle_neg_msg");
    }

    async fn handle_event(
        &self,
        event: Box<Event>,
        certified: bool,
    ) -> Result<HandlerResult, Error> {
        let response: RelayMessage;
        let eid: nostr::EventId = event.id();
        let event_kind = event.kind();
        if !certified && is_channel_message(&event) {
            log::debug!("Event is not certified");
            let outgoing = OutgoingMessage::new();
            return outgoing.send_challenge(CHALLENGE_MESSAGE).await;
        }
        if event_kind == nostr::Kind::EventDeletion {
            let filter = nostr::Filter::new().event(eid);
            self.db.delete(filter).await?;
            let content: String = event.content().to_string();
            response = RelayMessage::ok(eid, true, &content);
            let response_str: String = serde_json::to_string(&response)?;
            let ret = OperationData::new(response_str);
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
                let ret = OperationData::new(response_str);
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
        let ret = OperationData::new(response_str);
        Ok(HandlerResult::Event(ret))
    }
}

fn is_channel_message(event: &Event) -> bool {
    match event.kind() {
        nostr::Kind::ChannelMessage
        | nostr::Kind::ChannelCreation
        | nostr::Kind::ChannelMuteUser
        | nostr::Kind::ChannelMetadata => true,
        _ => false,
    }
}
