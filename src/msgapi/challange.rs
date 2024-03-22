use chrono::{Duration, Utc};
use nostr::event::Event;
use nostr::Kind;

pub struct Challenge {
    pub challenge_event: Event,
}

impl Challenge {
    pub async fn new(challenge_event: Event) -> Self {
        Challenge { challenge_event }
    }
    pub async fn time_check(&self) -> bool {
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
    pub async fn signature_check(&self) -> bool {
        let certified = self.challenge_event.verify_signature().is_err();
        if certified {
            return false;
        }
        true
    }
    pub async fn ClientAuthentication(&self) -> bool {
        let kind: nostr::Kind = self.challenge_event.kind;
        match kind {
            Kind::Authentication => {
                return true;
            }
            _ => {
                return false;
            }
        }
    }
    pub async fn relay_check(&self) -> bool {
        false
    }
}
