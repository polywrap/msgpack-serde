use crate::{
    error::EncodeError,
    format::{ExtensionType},
    BigInt, BigNumber, JSON,
};
use byteorder::{WriteBytesExt};
use core::hash::Hash;
use std::{
    collections::BTreeMap,
    io::{Cursor, Write as StdioWrite},
};

#[derive(Debug)]
pub struct WriteEncoder {
    pub buffer: Cursor<Vec<u8>>,
}

impl WriteEncoder {
    pub fn write_bigint(&mut self, value: &BigInt) -> Result<(), EncodeError> {
        self.write_string(&value.to_string())
            .map_err(|e| EncodeError::BigIntWriteError(e.to_string()))
    }

    pub fn write_bignumber(
        &mut self,
        value: &BigNumber,
    ) -> Result<(), EncodeError> {
        self.write_string(&value.to_string())
            .map_err(|e| EncodeError::BigIntWriteError(e.to_string()))
    }

    pub fn write_json(&mut self, value: &JSON::Value) -> Result<(), EncodeError> {
        let json_str = JSON::to_string(value)?;
        self.write_string(&json_str)
            .map_err(|e| EncodeError::JSONWriteError(e.to_string()))
    }
}

