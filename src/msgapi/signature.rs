use super::Error;
use k256::schnorr::Signature as KSignature;
use serde::{de::Visitor, Deserialize, Serialize};
use std::{fmt, ops::Deref};

#[derive(Clone, Debug)]
pub struct Signature(pub KSignature);

impl Signature {
    #[allow(dead_code)]
    pub fn as_hex_string(&self) -> String {
        hex::encode(self.0.to_bytes())
    }

    #[allow(dead_code)]
    pub fn try_from_hex_string(v: &str) -> Result<Signature, Error> {
        let vec: Vec<u8> = hex::decode(v)?;
        Ok(Signature(KSignature::try_from(&*vec)?))
    }

    pub fn try_from_vec_u8(v: Vec<u8>) -> Result<Signature, Error> {
        Ok(Signature(KSignature::try_from(&*v)?))
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0.to_bytes()))
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(SignatureVisitor)
    }
}

impl Deref for Signature {
    type Target = KSignature;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct SignatureVisitor;

impl Visitor<'_> for SignatureVisitor {
    type Value = Signature;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sig hexadecimal string representing 64 bytes")
    }

    fn visit_str<E>(self, v: &str) -> Result<Signature, E>
    where
        E: serde::de::Error,
    {
        let vec: Vec<u8> =
            hex::decode(v).map_err(|e| serde::de::Error::custom(format!("{}", e)))?;

        if vec.len() != 64 {
            return Err(serde::de::Error::custom("Signature is not 64 bytes long"));
        }

        let ksig: KSignature =
            KSignature::try_from(&*vec).map_err(|e| serde::de::Error::custom(format!("{}", e)))?;
        Ok(Signature(ksig))
    }
}
