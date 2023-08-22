#[allow(irrefutable_let_patterns)]

mod de;
pub mod error;
pub use error::*;
mod format;
mod ser;
pub mod wrappers;

pub use bigdecimal::BigDecimal as BigNumber;
pub use serde_json as JSON;
pub use std::collections::BTreeMap as Map;
pub use serde_bytes as bytes;
pub use num_bigint::{BigInt, ParseBigIntError};
pub use wrappers::polywrap_bigint::BigIntWrapper;
pub use wrappers::polywrap_json::JSONString;

pub use crate::de::{from_slice, Deserializer};
pub use ser::{to_vec, Serializer};
