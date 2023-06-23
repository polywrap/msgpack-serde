use std::{
    fmt::{self},
    str::FromStr,
};

use num_bigint::BigInt;
use serde::{de::Visitor, Deserialize, Serialize, Serializer, Deserializer};

#[derive(Debug, PartialEq)]
pub struct BigIntWrapper(pub BigInt);

pub fn serialize<S>(x: &BigInt, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&x.to_string())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<BigInt, D::Error>
where
  D: Deserializer<'de>,
{
  Ok(deserializer.deserialize_str(BigIntStrVisitor)?.0)
}

impl Serialize for BigIntWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

struct BigIntStrVisitor;

impl<'de> Visitor<'de> for BigIntStrVisitor {
    type Value = BigIntWrapper;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a BigInt string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let big_int = BigInt::from_str(v).map_err(|e| {
            serde::de::Error::custom(format!("Error parsing BigInt: {e}"))
        })?;

        Ok(BigIntWrapper(big_int))
    }
}

impl<'a> Deserialize<'a> for BigIntWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        deserializer.deserialize_str(BigIntStrVisitor)
    }
}
