use std::{
  fmt::{self},
};

use serde_json::Value;
use serde::{de::Visitor, Deserialize, Serialize, Serializer, Deserializer};

#[derive(PartialEq, Debug, Clone)]
pub struct JSON(pub Value);

pub fn serialize<S>(x: &Value, s: S) -> Result<S::Ok, S::Error>
where
  S: Serializer,
{
  s.serialize_str(&x.to_string())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<serde_json::Value, D::Error>
where
  D: Deserializer<'de>,
{
  Ok(deserializer.deserialize_str(JSONStrVisitor)?.0)
}

impl Serialize for JSON {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
      S: serde::Serializer,
  {
      serializer.serialize_str(&self.0.to_string())
  }
}

struct JSONStrVisitor;

impl<'de> Visitor<'de> for JSONStrVisitor {
  type Value = JSON;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      formatter.write_str("a JSON string")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
  where
      E: serde::de::Error,
  {
      let big_int = serde_json::from_str(v).map_err(|e| {
          serde::de::Error::custom(format!("Error parsing JSON: {e}"))
      })?;

      Ok(JSON(big_int))
  }
}

impl<'a> Deserialize<'a> for JSON {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
      D: serde::Deserializer<'a>,
  {
      deserializer.deserialize_str(JSONStrVisitor)
  }
}