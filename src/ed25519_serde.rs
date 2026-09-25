//! Serde compatibility for the project's legacy Ed25519-bearing data types.
//!
//! The privacy/custody suites pin `ed25519-dalek` without its optional Serde
//! feature. These adapters preserve the crate's former wire representation for
//! unrelated transaction, wallet, and PBFT records without broadening the
//! reviewed cryptographic feature set.

use std::fmt;

use ed25519_dalek::{Signature, VerifyingKey};
use serde::de::{self, SeqAccess, Visitor};
use serde::ser::SerializeTuple;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub(crate) mod signature {
    use super::*;

    pub(crate) fn serialize<S>(value: &Signature, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tuple = serializer.serialize_tuple(64)?;
        for byte in value.to_bytes() {
            tuple.serialize_element(&byte)?;
        }
        tuple.end()
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Signature, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SignatureVisitor;

        impl<'de> Visitor<'de> for SignatureVisitor {
            type Value = Signature;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("bytestring of length 64")
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut bytes = [0_u8; 64];
                for (index, byte) in bytes.iter_mut().enumerate() {
                    *byte = sequence
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(index, &self))?;
                }
                Ok(Signature::from_bytes(&bytes))
            }
        }

        deserializer.deserialize_tuple(64, SignatureVisitor)
    }
}

pub(crate) mod optional_signature {
    use super::*;

    struct SignatureRef<'a>(&'a Signature);

    impl Serialize for SignatureRef<'_> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            super::signature::serialize(self.0, serializer)
        }
    }

    struct SignatureValue(Signature);

    impl<'de> Deserialize<'de> for SignatureValue {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            super::signature::deserialize(deserializer).map(Self)
        }
    }

    pub(crate) fn serialize<S>(value: &Option<Signature>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(signature) => serializer.serialize_some(&SignatureRef(signature)),
            None => serializer.serialize_none(),
        }
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Option<Signature>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<SignatureValue>::deserialize(deserializer).map(|value| value.map(|value| value.0))
    }
}

pub(crate) mod verifying_key {
    use super::*;

    pub(crate) fn serialize<S>(value: &VerifyingKey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(value.as_bytes())
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<VerifyingKey, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VerifyingKeyVisitor;

        impl<'de> Visitor<'de> for VerifyingKeyVisitor {
            type Value = VerifyingKey;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an Ed25519 verifying key")
            }

            fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                VerifyingKey::try_from(bytes).map_err(E::custom)
            }

            fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut bytes = [0_u8; 32];
                for (index, byte) in bytes.iter_mut().enumerate() {
                    *byte = sequence
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(index, &"expected 32 bytes"))?;
                }
                if sequence.next_element::<u8>()?.is_some() {
                    return Err(de::Error::invalid_length(33, &"expected 32 bytes"));
                }
                VerifyingKey::from_bytes(&bytes).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_bytes(VerifyingKeyVisitor)
    }
}

pub(crate) mod optional_verifying_key {
    use super::*;

    struct VerifyingKeyRef<'a>(&'a VerifyingKey);

    impl Serialize for VerifyingKeyRef<'_> {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            super::verifying_key::serialize(self.0, serializer)
        }
    }

    struct VerifyingKeyValue(VerifyingKey);

    impl<'de> Deserialize<'de> for VerifyingKeyValue {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            super::verifying_key::deserialize(deserializer).map(Self)
        }
    }

    pub(crate) fn serialize<S>(
        value: &Option<VerifyingKey>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(key) => serializer.serialize_some(&VerifyingKeyRef(key)),
            None => serializer.serialize_none(),
        }
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Option<VerifyingKey>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Option::<VerifyingKeyValue>::deserialize(deserializer)
            .map(|value| value.map(|value| value.0))
    }
}
