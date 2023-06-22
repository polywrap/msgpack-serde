//! Errors returned from I/O `Write` and `Read` operations
use thiserror::Error;

use crate::format::Format;

/// Errors from encoding data
#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("{0}")]
    NilWriteError(String),

    #[error("{0}")]
    FormatWriteError(String),

    #[error("{0}")]
    BooleanWriteError(String),

    #[error("{0}")]
    BinWriteError(String),

    #[error("{0}")]
    BigIntWriteError(String),

    #[error("{0}")]
    JSONWriteError(String),

    #[error("{0}")]
    Float32WriteError(String),

    #[error("{0}")]
    Float64WriteError(String),

    #[error("{0}")]
    Uint8WriteError(String),

    #[error("{0}")]
    Uint16WriteError(String),

    #[error("{0}")]
    Uint32WriteError(String),

    #[error("{0}")]
    Int8WriteError(String),

    #[error("{0}")]
    Int16WriteError(String),

    #[error("{0}")]
    Int32WriteError(String),

    #[error("{0}")]
    StrWriteError(String),

    #[error("{0}")]
    TypeWriteError(String),

    #[error("{0}")]
    IOError(String),
}

impl From<std::io::Error> for EncodeError {
    fn from(e: std::io::Error) -> Self {
        EncodeError::IOError(e.to_string())
    }
}

impl From<serde_json::Error> for EncodeError {
    fn from(e: serde_json::Error) -> EncodeError {
        EncodeError::JSONWriteError(e.to_string())
    }
}

pub fn get_error_message(format: Format) -> String {
    match format {
        Format::Nil => "Found 'nil'.".to_string(),
        Format::Reserved => "Found 'reserved'.".to_string(),
        Format::False | Format::True => "Found 'bool'.".to_string(),
        Format::Bin8 => "Found 'BIN8'.".to_string(),
        Format::Bin16 => "Found 'BIN16'.".to_string(),
        Format::Bin32 => "Found 'BIN32'.".to_string(),
        Format::Ext8 => "Found 'EXT8'.".to_string(),
        Format::Ext16 => "Found 'EXT16'.".to_string(),
        Format::Ext32 => "Found 'EXT32'.".to_string(),
        Format::Float32 => "Found 'float32'.".to_string(),
        Format::Float64 => "Found 'float64'.".to_string(),
        Format::NegativeFixInt(_) | Format::PositiveFixInt(_) => "Found 'int'.".to_string(),
        Format::Uint8 => "Found 'uint8'.".to_string(),
        Format::Uint16 => "Found 'uint16'.".to_string(),
        Format::Uint32 => "Found 'uint32'.".to_string(),
        Format::Uint64 => "Found 'uint64'.".to_string(),
        Format::Int8 => "Found 'int8'.".to_string(),
        Format::Int16 => "Found 'int16'.".to_string(),
        Format::Int32 => "Found 'int32'.".to_string(),
        Format::Int64 => "Found 'int64'.".to_string(),
        Format::FixExt1 => "Found 'FIXEXT1'.".to_string(),
        Format::FixExt2 => "Found 'FIXEXT2'.".to_string(),
        Format::FixExt4 => "Found 'FIXEXT4'.".to_string(),
        Format::FixExt8 => "Found 'FIXEXT8'.".to_string(),
        Format::FixExt16 => "Found 'FIXEXT16'.".to_string(),
        Format::FixStr(_) | Format::Str8 | Format::Str16 | Format::Str32 => "Found 'string'.".to_string(),
        Format::FixArray(_) | Format::Array16 | Format::Array32 => "Found 'array'.".to_string(),
        Format::FixMap(_) | Format::Map16 | Format::Map32 => "Found 'map'.".to_string(),
    }
}

use serde::{de, ser};
use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Message(String),
    Eof,
    Syntax,
    ExpectedBoolean(String),
    ExpectedUInteger(String),
    ExpectedInteger(String),
    ExpectedBytes(String),
    ExpectedFloat(String),
    ExpectedChar(String),
    ExpectedString(String),
    ExpectedNull(String),
    ExpectedArray(String),
    ExpectedArrayComma(String),
    ExpectedArrayEnd(String),
    ExpectedMap(String),
    ExpectedExt(String),
    ExpectedMapColon(String),
    ExpectedMapComma(String),
    ExpectedMapEnd(String),
    ExpectedEnum(String),
    TrailingCharacters,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl From<EncodeError> for Error {
    fn from(value: EncodeError) -> Self {
        Error::Message(value.to_string())
    }
}

impl From<std::io::Error> for Error {
  fn from(value: std::io::Error) -> Self {
      Error::Message(value.to_string())
  }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => write!(f, "{}", msg),
            Error::Eof => f.write_str("unexpected end of input"),
            /* and so forth */
            _ => unimplemented!(),
        }
    }
}

impl std::error::Error for Error {}
