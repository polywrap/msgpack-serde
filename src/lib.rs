// Copyright 2018 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

mod de;
mod error;
mod ser;
mod format;
mod context;
mod write;
mod write_encoder;
mod data_view;

pub use num_bigint::BigInt;
pub use bigdecimal::BigDecimal as BigNumber;
pub use serde_json as JSON;
pub use std::collections::BTreeMap as Map;

pub use crate::de::{from_str, Deserializer};
pub use crate::ser::{to_vec, Serializer};
