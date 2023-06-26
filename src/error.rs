use std::fmt::Display;

use serde::{ser, de};

use crate::format::Format;

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

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("`{0}`")]
    Message(String),
    #[error("EOF error")]
    Eof,
    #[error("Syntax Error")]
    Syntax,
    #[error("Expected Boolean: `{0}`")]
    ExpectedBoolean(String),
    #[error("Expected Unsigned Integer: `{0}`")]
    ExpectedUInteger(String),
    #[error("Expected Integer: `{0}`")]
    ExpectedInteger(String),
    #[error("Expected Bytes: `{0}`")]
    ExpectedBytes(String),
    #[error("Expected Float: `{0}`")]
    ExpectedFloat(String),
    #[error("Expected Char: `{0}`")]
    ExpectedChar(String),
    #[error("Expected String: `{0}`")]
    ExpectedString(String),
    #[error("Expected Null: `{0}`")]
    ExpectedNull(String),
    #[error("Expected Array: `{0}`")]
    ExpectedArray(String),
    #[error("Expected Map: `{0}`")]
    ExpectedMap(String),
    #[error("Expected Ext: `{0}`")]
    ExpectedExt(String),
    #[error("Expected Enum: `{0}`")]
    ExpectedEnum(String),
    #[error("Trailing characters in deserialization")]
    TrailingCharacters,
}

impl From<std::io::Error> for Error {
  fn from(value: std::io::Error) -> Self {
      Error::Message(value.to_string())
  }
}

impl ser::Error for Error {
  fn custom<T: Display>(msg: T) -> Self {
      Error::Message(msg.to_string())
  }
}

impl de::Error for Error {
  fn custom<T: Display>(msg: T) -> Self {
      Error::Message(msg.to_string())
  }
}
