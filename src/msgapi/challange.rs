use chrono::{Duration, Utc};
use nostr::event::Event;
use nostr::Kind;

pub struct Challenge<'a> {
    pub challenge_event: &'a Box<Event>,
}

impl<'a> Challenge<'a> {
    pub fn new(challenge_event: &'a Box<Event>) -> Self {
        Challenge { challenge_event }
    }
    pub fn time_check(&self) -> bool {
        let crate_time: nostr::Timestamp = self.challenge_event.created_at;
        let now = Utc::now();
        let ten_minutes = Duration::try_minutes(10);
        let ten_minutes_ago = now - ten_minutes.unwrap();
        let ten_minutes_ago_timestamp = nostr::Timestamp::from(ten_minutes_ago.timestamp() as u64);
        if crate_time < ten_minutes_ago_timestamp {
            return false;
        }
        true
    }
    pub fn signature_check(&self) -> bool {
        let certified = self.challenge_event.verify_signature().is_err();
        if certified {
            return false;
        }
        true
    }
    pub fn client_authentication(&self) -> bool {
        let kind: nostr::Kind = self.challenge_event.kind;
        matches!(kind, Kind::Authentication)
    }
    pub fn relay_check(&self) -> bool {
        true
    }
}
