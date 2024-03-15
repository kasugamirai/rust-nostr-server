use std::fmt;

use super::{Id, PublicKey};
use serde::{
    de::{SeqAccess, Visitor},
    ser::SerializeSeq,
    Deserialize, Deserializer, Serialize,
};

#[derive(Clone, Debug)]
pub enum Tag {
    Event {
        id: Id,
        recommended_relay_url: Option<String>,
        marker: Option<String>,
    },
    Pubkey {
        pubkey: PublicKey,
        recommended_relay_url: Option<String>,
        petname: Option<String>,
    },
    Relay(String),
    Challenge(String),
    Subject(String),
    Empty,
    /// Any other tag
    Other {
        /// The tag name
        tag: String,
        /// The subsequent fields
        data: Vec<String>,
    },
}

impl Tag {
    #[allow(dead_code)]
    pub fn tagname(&self) -> String {
        match self {
            Tag::Event { .. } => "e".to_string(),
            Tag::Subject(_) => "subject".to_string(),
            Tag::Pubkey { .. } => "p".to_string(),
            Tag::Relay(_) => "relay".to_string(),
            Tag::Challenge(_) => "challenge".to_string(),
            Tag::Empty => panic!("empty tags have no tagname"),
            Tag::Other { tag, .. } => tag.to_owned(),
        }
    }
}

impl Serialize for Tag {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Tag::Event {
                id,
                recommended_relay_url,
                marker,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("e")?;
                seq.serialize_element(id)?;
                if let Some(rru) = recommended_relay_url {
                    seq.serialize_element(rru)?;
                } else if let Some(m) = marker {
                    seq.serialize_element(m)?;
                } else if marker.is_some() {
                    seq.serialize_element("")?;
                }
                seq.end()
            }
            Tag::Pubkey {
                pubkey,
                recommended_relay_url,
                petname,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("p")?;
                seq.serialize_element(pubkey)?;
                if let Some(rru) = recommended_relay_url {
                    seq.serialize_element(rru)?;
                } else if petname.is_some() {
                    seq.serialize_element("")?;
                }
                if let Some(pn) = petname {
                    seq.serialize_element(pn)?;
                }
                seq.end()
            }
            Tag::Challenge(challenge) => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("challenge")?;
                seq.serialize_element(challenge)?;
                seq.end()
            }
            Tag::Relay(relay) => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("relay")?;
                seq.serialize_element(relay)?;
                seq.end()
            }
            Tag::Subject(subject) => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("subject")?;
                seq.serialize_element(subject)?;
                seq.end()
            }
            Tag::Other { tag, data } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element(tag)?;
                for s in data.iter() {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            Tag::Empty => {
                let seq = serializer.serialize_seq(Some(0))?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for Tag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(TagVisitor)
    }
}

struct TagVisitor;

impl<'de> Visitor<'de> for TagVisitor {
    type Value = Tag;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Tag, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let tagname: &str = match seq.next_element()? {
            Some(e) => e,
            None => return Ok(Tag::Empty),
        };
        match tagname {
            "e" => {
                let id: Id = match seq.next_element()? {
                    Some(id) => id,
                    None => {
                        return Ok(Tag::Other {
                            tag: tagname.to_string(),
                            data: vec![],
                        });
                    }
                };
                let recommended_relay_url: Option<String> = seq.next_element()?;
                let marker: Option<String> = seq.next_element()?;
                Ok(Tag::Event {
                    id,
                    recommended_relay_url,
                    marker,
                })
            }
            "p" => {
                let pubkey: PublicKey = match seq.next_element()? {
                    Some(p) => p,
                    None => {
                        return Ok(Tag::Other {
                            tag: tagname.to_string(),
                            data: vec![],
                        });
                    }
                };
                let recommended_relay_url: Option<String> = seq.next_element()?;
                let petname: Option<String> = seq.next_element()?;
                Ok(Tag::Pubkey {
                    pubkey,
                    recommended_relay_url,
                    petname,
                })
            }
            "subject" => {
                let sub = match seq.next_element()? {
                    Some(s) => s,
                    None => {
                        return Ok(Tag::Other {
                            tag: tagname.to_string(),
                            data: vec![],
                        });
                    }
                };
                Ok(Tag::Subject(sub))
            }
            "relay" => {
                let relay = match seq.next_element()? {
                    Some(s) => s,
                    None => return Ok(Tag::Relay("".to_string())),
                };
                Ok(Tag::Relay(relay))
            }
            "challenge" => {
                let challenge = match seq.next_element()? {
                    Some(s) => s,
                    None => return Ok(Tag::Challenge("".to_string())),
                };
                Ok(Tag::Challenge(challenge))
            }
            _ => {
                let mut data = Vec::new();
                loop {
                    match seq.next_element()? {
                        None => {
                            return Ok(Tag::Other {
                                tag: tagname.to_string(),
                                data,
                            })
                        }
                        Some(s) => data.push(s),
                    }
                }
            }
        }
    }
}
