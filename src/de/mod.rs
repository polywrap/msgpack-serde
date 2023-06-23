mod array;
mod map;
mod _enum;

use crate::{
  error::{get_error_message, Error, Result},
  format::{ExtensionType, Format},
};
use byteorder::{BigEndian, ReadBytesExt};
use serde::de::{self, Deserialize, IntoDeserializer, Visitor};
use std::io::{Cursor, Read};

use array::ArrayReadAccess;
use map::MapReadAccess;

pub struct Deserializer {
  pub buffer: Cursor<Vec<u8>>,
}

impl Default for Deserializer {
  fn default() -> Self {
      Self {
          buffer: Cursor::new(vec![]),
      }
  }
}

impl Deserializer {
  #[allow(clippy::should_implement_trait)]
  pub fn from_slice(buffer: &[u8]) -> Self {
      Deserializer {
          buffer: Cursor::new(buffer.to_vec()),
      }
  }
}

pub fn from_slice<'a, T>(buffer: &'a [u8]) -> Result<T>
where
  T: Deserialize<'a>,
{
  let mut deserializer = Deserializer::from_slice(buffer);
  let t = T::deserialize(&mut deserializer)?;
  if deserializer.buffer.position() as usize
      == deserializer.buffer.get_ref().len()
  {
      Ok(t)
  } else {
      Err(Error::TrailingCharacters)
  }
}

impl Deserializer {
  fn peek_format(&mut self) -> Result<Format> {
      let position = self.buffer.position();
      let format = Format::get_format(self)?;
      self.buffer.set_position(position);

      Ok(format)
  }

  fn read_ext_length_and_type(&mut self) -> Result<(u32, ExtensionType)> {
      let format = Format::get_format(self)?;
      let byte_length = match format {
          Format::FixExt1 => 1,
          Format::FixExt2 => 2,
          Format::FixExt4 => 4,
          Format::FixExt8 => 8,
          Format::FixExt16 => 16,
          Format::Ext8 => ReadBytesExt::read_u8(self)? as u32,
          Format::Ext16 => ReadBytesExt::read_u16::<BigEndian>(self)? as u32,
          Format::Ext32 => ReadBytesExt::read_u32::<BigEndian>(self)?,
          err_f => {
              let formatted_err = format!(
                  "Property must be of type 'ext generic map'. {}",
                  get_error_message(err_f)
              );
              return Err(Error::ExpectedExt(formatted_err));
          }
      };

      let ext_type = ReadBytesExt::read_u8(self)?;

      Ok((byte_length, ext_type.try_into()?))
  }

  fn read_array_length(&mut self) -> Result<u32> {
      let next_format = self.peek_format()?;

      if let Format::Nil = next_format {
          return Ok(0);
      }

      match Format::get_format(self)? {
          Format::FixArray(len) => Ok(len as u32),
          Format::Array16 => {
              Ok(ReadBytesExt::read_u16::<BigEndian>(self)? as u32)
          }
          Format::Array32 => Ok(ReadBytesExt::read_u32::<BigEndian>(self)?),
          Format::Nil => Ok(0),
          err_f => {
              let formatted_err = format!(
                  "Property must be of type 'array'. {}",
                  get_error_message(err_f)
              );
              Err(Error::ExpectedArray(formatted_err))
          }
      }
  }

  fn get_bytes(&mut self, n_bytes_to_read: u64) -> Result<Vec<u8>> {
      let mut buf = vec![];
      let mut chunk = self.take(n_bytes_to_read);
      match chunk.read_to_end(&mut buf) {
          Ok(_n) => Ok(buf),
          Err(e) => Err(Error::Message(e.to_string())),
      }
  }

  fn read_string_length(&mut self) -> Result<u32> {
      let next_format = self.peek_format()?;

      if let Format::Nil = next_format {
          return Ok(0);
      }

      match Format::get_format(self)? {
          Format::FixStr(len) => Ok(len as u32),
          Format::FixArray(len) => Ok(len as u32),
          Format::Str8 => Ok(ReadBytesExt::read_u8(self)? as u32),
          Format::Str16 => {
              Ok(ReadBytesExt::read_u16::<BigEndian>(self)? as u32)
          }
          Format::Str32 => Ok(ReadBytesExt::read_u32::<BigEndian>(self)?),
          Format::Nil => Ok(0),
          err_f => {
              let formatted_err = format!(
                  "Property must be of type 'string'. {}",
                  get_error_message(err_f)
              );
              Err(Error::ExpectedString(formatted_err))
          }
      }
  }

  fn parse_string(&mut self) -> Result<String> {
      let str_len = self.read_string_length()?;
      let bytes = self.get_bytes(str_len as u64)?;
      match String::from_utf8(bytes) {
          Ok(s) => Ok(s),
          Err(e) => Err(Error::Message(e.to_string())),
      }
  }

  fn read_map_length(&mut self) -> Result<u32> {
      let next_format = self.peek_format()?;

      if let Format::Nil = next_format {
          return Ok(0);
      }

      match Format::get_format(self)? {
          Format::FixMap(len) => Ok(len as u32),
          Format::Map16 => {
              Ok(ReadBytesExt::read_u16::<BigEndian>(self)? as u32)
          }
          Format::Map32 => Ok(ReadBytesExt::read_u32::<BigEndian>(self)?),
          Format::Nil => Ok(0),
          err_f => {
              let formatted_err = format!(
                  "Property must be of type 'map'. {}",
                  get_error_message(err_f)
              );
              Err(Error::ExpectedMap(formatted_err))
          }
      }
  }

  fn read_bytes_length(&mut self) -> Result<u32> {
      let next_format = self.peek_format()?;

      if let Format::Nil = next_format {
          return Ok(0);
      }

      match Format::get_format(self)? {
          Format::FixArray(len) => Ok(len as u32),
          Format::Bin8 => Ok(ReadBytesExt::read_u8(self)? as u32),
          Format::Bin16 => {
              Ok(ReadBytesExt::read_u16::<BigEndian>(self)? as u32)
          }
          Format::Bin32 => Ok(ReadBytesExt::read_u32::<BigEndian>(self)?),
          Format::Nil => Ok(0),
          err_f => {
              let formatted_err = format!(
                  "Property must be of type 'bytes'. {}",
                  get_error_message(err_f)
              );
              Err(Error::ExpectedBytes(formatted_err))
          }
      }
  }

  fn parse_unsigned(&mut self) -> Result<u64> {
      let f = Format::get_format(self)?;
      let prefix = f.to_u8();
      if Format::is_positive_fixed_int(prefix) {
          return Ok(prefix as u64);
      } else if Format::is_negative_fixed_int(prefix) {
          let formatted_err = format!(
              "unsigned integer cannot be negative. {}",
              get_error_message(f)
          );

          return Err(Error::ExpectedUInteger(formatted_err));
      }

      match f {
          Format::Uint8 => Ok(ReadBytesExt::read_u8(self)? as u64),
          Format::Uint16 => {
              Ok(ReadBytesExt::read_u16::<BigEndian>(self)? as u64)
          }
          Format::Uint32 => {
              Ok(ReadBytesExt::read_u32::<BigEndian>(self)? as u64)
          }
          Format::Uint64 => Ok(ReadBytesExt::read_u64::<BigEndian>(self)?),
          Format::Int8 => {
              let int8 = ReadBytesExt::read_i8(self)?;

              if int8 >= 0 {
                  return Ok(int8 as u64);
              }

              let formatted_err = format!(
                  "unsigned integer cannot be negative. {}",
                  get_error_message(f)
              );
              Err(Error::ExpectedUInteger(formatted_err))
          }
          Format::Int16 => {
              let int16 = ReadBytesExt::read_i16::<BigEndian>(self)?;

              if int16 >= 0 {
                  return Ok(int16 as u64);
              }

              let formatted_err = format!(
                  "unsigned integer cannot be negative. {}",
                  get_error_message(f)
              );
              Err(Error::ExpectedUInteger(formatted_err))
          }
          Format::Int32 => {
              let int32 = ReadBytesExt::read_i32::<BigEndian>(self)?;

              if int32 >= 0 {
                  return Ok(int32 as u64);
              }

              let formatted_err = format!(
                  "unsigned integer cannot be negative. {}",
                  get_error_message(f)
              );
              Err(Error::ExpectedUInteger(formatted_err))
          }
          Format::Int64 => {
              let int64 = ReadBytesExt::read_i64::<BigEndian>(self)?;

              if int64 >= 0 {
                  return Ok(int64 as u64);
              }

              let formatted_err = format!(
                  "unsigned integer cannot be negative. {}",
                  get_error_message(f)
              );
              Err(Error::ExpectedUInteger(formatted_err))
          }

          err_f => {
              let formatted_err = format!(
                  "Property must be of type 'uint'. {}",
                  get_error_message(err_f)
              );
              Err(Error::ExpectedUInteger(formatted_err))
          }
      }
  }

  fn parse_signed(&mut self) -> Result<i64> {
      let f = Format::get_format(self)?;
      let prefix = f.to_u8();
      if Format::is_positive_fixed_int(prefix) {
          Ok(prefix as i64)
      } else if Format::is_negative_fixed_int(prefix) {
          Ok((prefix as i8) as i64)
      } else {
          match f {
              Format::Int8 => Ok(ReadBytesExt::read_i8(self)? as i64),
              Format::Int16 => {
                  Ok(ReadBytesExt::read_i16::<BigEndian>(self)? as i64)
              }
              Format::Int32 => {
                  Ok(ReadBytesExt::read_i32::<BigEndian>(self)? as i64)
              }
              Format::Int64 => Ok(ReadBytesExt::read_i64::<BigEndian>(self)?),
              Format::Uint8 => Ok(ReadBytesExt::read_u8(self)? as i64),
              Format::Uint16 => {
                  Ok(ReadBytesExt::read_u16::<BigEndian>(self)? as i64)
              }
              Format::Uint32 => {
                  Ok(ReadBytesExt::read_u32::<BigEndian>(self)? as i64)
              }
              Format::Uint64 => {
                  let v = ReadBytesExt::read_u64::<BigEndian>(self)?;

                  if v <= i64::MAX as u64 {
                      Ok(v as i64)
                  } else {
                      let formatted_err = format!(
                          "integer overflow: value = {}; bits = 64",
                          v
                      );
                      Err(Error::Message(formatted_err))
                  }
              }
              err_f => {
                  let formatted_err = format!(
                      "Property must be of type 'int'. {}",
                      get_error_message(err_f)
                  );
                  Err(Error::ExpectedInteger(formatted_err))
              }
          }
      }
  }
}

impl<'de, 'a> de::Deserializer<'de> for &'a mut Deserializer {
  type Error = Error;

  fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      match self.peek_format()? {
          Format::PositiveFixInt(_)
          | Format::NegativeFixInt(_)
          | Format::Int8 => self.deserialize_i8(visitor),
          Format::FixMap(_) | Format::Map16 | Format::Map32 => todo!(),
          Format::FixArray(_) | Format::Array16 | Format::Array32 => {
              self.deserialize_seq(visitor)
          }
          Format::FixStr(_)
          | Format::Str8
          | Format::Str16
          | Format::Str32 => self.deserialize_string(visitor),
          Format::Nil => self.deserialize_unit(visitor),
          Format::Reserved => todo!(),
          Format::False | Format::True => self.deserialize_bool(visitor),
          Format::Bin8 | Format::Bin16 | Format::Bin32 => {
              self.deserialize_bytes(visitor)
          }
          Format::Float32 => self.deserialize_f32(visitor),
          Format::Float64 => self.deserialize_f64(visitor),
          Format::Uint8 => self.deserialize_u8(visitor),
          Format::Uint16 => self.deserialize_u16(visitor),
          Format::Uint32 => self.deserialize_u32(visitor),
          Format::Uint64 => self.deserialize_u64(visitor),
          Format::Int16 => self.deserialize_i16(visitor),
          Format::Int32 => self.deserialize_i32(visitor),
          Format::Int64 => self.deserialize_i64(visitor),
          Format::FixExt1
          | Format::FixExt2
          | Format::FixExt4
          | Format::FixExt8
          | Format::FixExt16
          | Format::Ext8
          | Format::Ext16
          | Format::Ext32 => todo!(),
      }
  }

  fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      match Format::get_format(self)? {
          Format::True => visitor.visit_bool(true),
          Format::False => visitor.visit_bool(false),
          err_f => {
              let formatted_err = format!(
                  "Property must be of type 'bool'. {}",
                  get_error_message(err_f)
              );
              Err(Error::ExpectedBoolean(formatted_err))
          }
      }
  }

  fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      let v = self.parse_signed()?;
      if v <= i8::MAX as i64 && v >= i8::MIN as i64 {
          visitor.visit_i8(v as i8)
      } else {
          let formatted_err =
              format!("integer overflow: value = {}; bits = 8", v);
          Err(Error::Message(formatted_err))
      }
  }

  fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      let v = self.parse_signed()?;
      if v <= i16::MAX as i64 && v >= i16::MIN as i64 {
          visitor.visit_i16(v as i16)
      } else {
          let formatted_err =
              format!("integer overflow: value = {}; bits = 16", v);
          Err(Error::Message(formatted_err))
      }
  }

  fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      let v = self.parse_signed()?;
      if v <= i32::MAX as i64 && v >= i32::MIN as i64 {
          visitor.visit_i32(v as i32)
      } else {
          let formatted_err =
              format!("integer overflow: value = {}; bits = 32", v);
          Err(Error::Message(formatted_err))
      }
  }

  fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      visitor.visit_i64(self.parse_signed()?)
  }

  fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      let v = self.parse_unsigned()?;

      if v <= u8::MAX as u64 && v >= u8::MIN as u64 {
          visitor.visit_u8(v as u8)
      } else {
          let formatted_err =
              format!("unsigned integer overflow: value = {}; bits = 8", v);
          Err(Error::Message(formatted_err))
      }
  }

  fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      let v = self.parse_unsigned()?;

      if v <= u16::MAX as u64 && v >= u16::MIN as u64 {
          visitor.visit_u16(v as u16)
      } else {
          let formatted_err =
              format!("unsigned integer overflow: value = {}; bits = 16", v);
          Err(Error::Message(formatted_err))
      }
  }

  fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      let v = self.parse_unsigned()?;

      if v <= u32::MAX as u64 && v >= u32::MIN as u64 {
          visitor.visit_u32(v as u32)
      } else {
          let formatted_err =
              format!("unsigned integer overflow: value = {}; bits = 32", v);
          Err(Error::Message(formatted_err))
      }
  }

  fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      visitor.visit_u64(self.parse_unsigned()?)
  }

  fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      match Format::get_format(self)? {
          Format::Float32 => {
              visitor.visit_f32(ReadBytesExt::read_f32::<BigEndian>(self)?)
          }
          err_f => {
              let formatted_err = format!(
                  "Property must be of type 'float32'. {}",
                  get_error_message(err_f)
              );
              Err(Error::ExpectedFloat(formatted_err))
          }
      }
  }

  fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      match Format::get_format(self)? {
          Format::Float64 => {
              visitor.visit_f64(ReadBytesExt::read_f64::<BigEndian>(self)?)
          }
          Format::Float32 => visitor
              .visit_f64(ReadBytesExt::read_f32::<BigEndian>(self)? as f64),
          err_f => {
              let formatted_err = format!(
                  "Property must be of type 'float64'. {}",
                  get_error_message(err_f)
              );
              Err(Error::ExpectedFloat(formatted_err))
          }
      }
  }

  fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      // TODO: maybe better implementation
      let str = self.parse_string()?;

      if str.len() == 1 {
          visitor.visit_char(str.chars().last().unwrap())
      } else {
          Err(Error::ExpectedChar(format!(
              "Expected char, found string: '{}'",
              str
          )))
      }
  }

  fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      self.deserialize_string(visitor)
  }

  fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      visitor.visit_string(self.parse_string()?)
  }

  fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      let bytes_len = self.read_bytes_length()?;
      let bytes = self.get_bytes(bytes_len as u64)?;
      visitor.visit_bytes(&bytes)
  }

  fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      let bytes_len = self.read_bytes_length()?;
      let bytes = self.get_bytes(bytes_len as u64)?;
      visitor.visit_byte_buf(bytes)
  }

  fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      match self.peek_format()? {
          Format::Nil => {
              Format::get_format(self)?;
              visitor.visit_none()
          }
          _ => visitor.visit_some(self),
      }
  }

  fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      match Format::get_format(self)? {
          Format::Nil => visitor.visit_unit(),
          format => {
              // TODO: better error messaging
              Err(Error::ExpectedNull(format!(
                  "Expected null, found format: {}",
                  format
              )))
          }
      }
  }

  fn deserialize_unit_struct<V>(
      self,
      _name: &'static str,
      visitor: V,
  ) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      self.deserialize_unit(visitor)
  }

  fn deserialize_newtype_struct<V>(
      self,
      _name: &'static str,
      visitor: V,
  ) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      visitor.visit_newtype_struct(self)
  }

  fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      let arr_len = self.read_array_length()?;
      visitor.visit_seq(ArrayReadAccess::new(self, arr_len))
  }

  fn deserialize_tuple<V>(self, _len: usize, _: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      todo!()
  }

  fn deserialize_tuple_struct<V>(
      self,
      _name: &'static str,
      _len: usize,
      _: V,
  ) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      todo!()
  }

  fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      let (_, ext_type) = self.read_ext_length_and_type()?;

      if let ExtensionType::GenericMap = ext_type {
          let ext_type: u8 = ext_type.into();
          let formatted_err = format!(
              "Extension must be of type 'ext generic map'. Found {ext_type}"
          );
          return Err(Error::ExpectedExt(formatted_err));
      }

      let map_len = self.read_map_length()?;

      visitor.visit_map(MapReadAccess::new(self, map_len))
  }

  fn deserialize_struct<V>(
      self,
      _name: &'static str,
      _fields: &'static [&'static str],
      visitor: V,
  ) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      let map_len = self.read_map_length()?;

      visitor.visit_map(MapReadAccess::new(self, map_len))
  }

  fn deserialize_enum<V>(
      self,
      _name: &'static str,
      variants: &'static [&'static str],
      visitor: V,
  ) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      match self.peek_format()? {
          Format::Uint8
          | Format::Uint16
          | Format::Uint32
          | Format::Uint64
          | Format::Int8
          | Format::Int16
          | Format::Int32
          | Format::Int64
          | Format::NegativeFixInt(_)
          | Format::PositiveFixInt(_) => {
              let index = self.parse_unsigned()?;
              let variant = variants.get(index as usize);

              if let Some(variant) = variant {
                  let variant = variant.to_string();
                  visitor.visit_enum(variant.into_deserializer())
              } else {
                  // TODO: better error handling
                  Err(Error::ExpectedUInteger("Expected enum variant as an unsigned integer".to_string()))
              }
          }
          Format::Str8
          | Format::Str16
          | Format::Str32
          | Format::FixStr(_) => {
              visitor.visit_enum(self.parse_string()?.into_deserializer())
          }
          format => Err(Error::Message(format!(
              "Expected valid enum variant, found: {}",
              format
          ))),
      }
  }

  fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      self.deserialize_str(visitor)
  }

  fn deserialize_ignored_any<V>(self, _: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
      todo!()
  }
}

impl Read for Deserializer {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
      self.buffer.read(&mut *buf)
  }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
  use std::{
      collections::{BTreeMap, HashMap},
      str::FromStr,
  };

  use num_bigint::BigInt;
  use serde_derive::Deserialize;

  use crate::{
      from_slice,
      wrappers::{polywrap_bigint::BigIntWrapper, polywrap_json::JSON},
  };

  #[test]
  fn test_read_empty_string() {
      let result: String = from_slice(&[160]).unwrap();
      assert_eq!("".to_string(), result);
  }

  #[test]
  fn test_read_string_5char() {
      let result: String =
          from_slice(&[165, 104, 101, 108, 108, 111]).unwrap();
      assert_eq!("hello".to_string(), result);
  }

  #[test]
  fn test_read_string_11char() {
      let result: String = from_slice(&[
          171, 104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100,
      ])
      .unwrap();
      assert_eq!("hello world".to_string(), result);
  }

  #[test]
  fn test_read_string_31char() {
      let result: String = from_slice(&[
          191, 45, 84, 104, 105, 115, 32, 115, 116, 114, 105, 110, 103, 32,
          99, 111, 110, 116, 97, 105, 110, 115, 32, 51, 49, 32, 99, 104, 97,
          114, 115, 45,
      ])
      .unwrap();
      assert_eq!("-This string contains 31 chars-".to_string(), result);
  }

  #[test]
  fn test_read_string_255char() {
      let result: String = from_slice(&[
          217, 255, 84, 104, 105, 115, 32, 105, 115, 32, 97, 32, 115, 116,
          114, 32, 56, 32, 115, 116, 114, 105, 110, 103, 32, 111, 102, 32,
          50, 53, 53, 32, 98, 121, 116, 101, 115, 32, 65, 67, 53, 51, 76,
          103, 120, 76, 76, 79, 75, 109, 48, 104, 102, 115, 80, 97, 49, 86,
          48, 110, 102, 77, 106, 88, 116, 110, 109, 107, 69, 116, 116, 114,
          117, 67, 80, 106, 99, 53, 49, 100, 116, 69, 77, 76, 82, 74, 73, 69,
          117, 49, 89, 111, 82, 71, 100, 57, 111, 88, 110, 77, 52, 67, 120,
          99, 73, 105, 84, 99, 57, 86, 50, 68, 110, 65, 105, 100, 90, 122,
          50, 50, 102, 111, 73, 122, 99, 51, 107, 113, 72, 66, 111, 88, 103,
          89, 115, 107, 101, 118, 102, 111, 74, 53, 82, 75, 89, 112, 53, 50,
          113, 118, 111, 68, 80, 117, 102, 85, 101, 98, 76, 107, 115, 70,
          108, 55, 97, 115, 116, 66, 78, 69, 110, 106, 80, 86, 85, 88, 50,
          101, 51, 79, 57, 79, 54, 86, 75, 101, 85, 112, 66, 48, 105, 105,
          72, 81, 88, 102, 122, 79, 79, 106, 84, 69, 75, 54, 88, 121, 54,
          107, 115, 52, 122, 65, 71, 50, 77, 54, 106, 67, 76, 48, 49, 102,
          108, 73, 74, 108, 120, 112, 108, 82, 88, 67, 86, 55, 32, 115, 97,
          100, 115, 97, 100, 115, 97, 100, 115, 97, 100, 97, 115, 100, 97,
          115, 97, 97, 97, 97, 97,
      ])
      .unwrap();
      assert_eq!(concat!("This is a str 8 string of 255 bytes ",
      "AC53LgxLLOKm0hfsPa1V0nfMjXtnmkEttruCPjc51dtEMLRJIEu1YoRGd9", "oXnM4CxcIiTc9V2DnAidZz22foIzc3kqHBoXgYskevfoJ5RK",
      "Yp52qvoDPufUebLksFl7astBNEnjPVUX2e3O9O6VKeUpB0iiHQXfzOOjTEK6Xy6ks4zAG2M6jCL01flIJlxplRXCV7 sadsadsadsadasdasaaaaa").to_string(), result);
  }

  #[test]
  fn test_read_array() {
      let result: Vec<i32> =
          from_slice(&[221, 0, 0, 0, 3, 1, 2, 206, 0, 8, 82, 65]).unwrap();
      let input_arr: Vec<i32> = vec![1, 2, 545345];
      assert_eq!(input_arr, result);
  }

  #[test]
  fn test_read_ext_map_8() {
      let result: BTreeMap<i32, Vec<i32>> =
          from_slice(&[199, 11, 1, 130, 1, 147, 3, 5, 9, 2, 147, 1, 4, 7])
              .unwrap();
      assert_eq!(result[&1], vec![3, 5, 9]);
      assert_eq!(result[&2], vec![1, 4, 7]);
  }

  #[test]
  fn test_read_ext_map_16() {
      let mut map2: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
      for i in 0..16 {
          map2.insert(i, vec![i, i + 1, i + 2]);
      }
      let result: BTreeMap<i32, Vec<i32>> = from_slice(&[
          199, 83, 1, 222, 0, 16, 0, 147, 0, 1, 2, 1, 147, 1, 2, 3, 2, 147,
          2, 3, 4, 3, 147, 3, 4, 5, 4, 147, 4, 5, 6, 5, 147, 5, 6, 7, 6, 147,
          6, 7, 8, 7, 147, 7, 8, 9, 8, 147, 8, 9, 10, 9, 147, 9, 10, 11, 10,
          147, 10, 11, 12, 11, 147, 11, 12, 13, 12, 147, 12, 13, 14, 13, 147,
          13, 14, 15, 14, 147, 14, 15, 16, 15, 147, 15, 16, 17,
      ])
      .unwrap();
      assert_eq!(map2, result);
  }

  #[test]
  fn test_read_ext_generic_map_nested() {
      let mut root_map: BTreeMap<String, BTreeMap<String, u8>> =
          BTreeMap::new();
      let mut sub_map: BTreeMap<String, u8> = BTreeMap::new();
      sub_map.insert("Hello".to_string(), 1);
      sub_map.insert("Heyo".to_string(), 50);
      root_map.insert("Nested".to_string(), sub_map);

      let result: BTreeMap<String, BTreeMap<String, u8>> = from_slice(&[
          199, 25, 1, 129, 166, 78, 101, 115, 116, 101, 100, 199, 14, 1, 130,
          165, 72, 101, 108, 108, 111, 1, 164, 72, 101, 121, 111, 50,
      ])
      .unwrap();
      assert_eq!(root_map, result);
  }

  #[test]
  fn test_read_ext_map_with_hashmap() {
      let result: HashMap<i32, Vec<i32>> =
          from_slice(&[199, 11, 1, 130, 1, 147, 3, 5, 9, 2, 147, 1, 4, 7])
              .unwrap();
      assert_eq!(result[&1], vec![3, 5, 9]);
      assert_eq!(result[&2], vec![1, 4, 7]);
  }

  #[test]
  fn test_read_bool_true() {
      let result: bool = from_slice(&[195]).unwrap();
      assert!(result);
  }

  #[test]
  fn test_read_bool_false() {
      let result: bool = from_slice(&[194]).unwrap();
      assert!(!result);
  }

  #[test]
  fn test_read_i8() {
      let result: i8 = from_slice(&[208, 128]).unwrap();
      assert_eq!(i8::MIN, result);
  }

  #[test]
  fn test_read_i16() {
      let result: i16 = from_slice(&[209, 128, 0]).unwrap();
      assert_eq!(i16::MIN, result);
  }

  #[test]
  fn test_read_i32() {
      let result: i32 = from_slice(&[210, 128, 0, 0, 0]).unwrap();
      assert_eq!(i32::MIN, result);
  }

  #[test]
  fn test_read_i64() {
      let result: i64 = from_slice(&[211, 128, 0, 0, 0, 0, 0, 0, 0]).unwrap();
      assert_eq!(i64::MIN, result);
  }

  #[test]
  fn test_read_u8() {
      let result: u8 = from_slice(&[204, 255]).unwrap();
      assert_eq!(u8::MAX, result);
  }

  #[test]
  fn test_read_u16() {
      let result: u16 = from_slice(&[205, 255, 255]).unwrap();
      assert_eq!(u16::MAX, result);
  }

  #[test]
  fn test_read_u32() {
      let result: u32 = from_slice(&[206, 255, 255, 255, 255]).unwrap();
      assert_eq!(u32::MAX, result);
  }

  #[test]
  fn test_read_u64() {
      let result: u64 =
          from_slice(&[207, 255, 255, 255, 255, 255, 255, 255, 255]).unwrap();
      assert_eq!(u64::MAX, result);
  }

  #[test]
  fn test_fixarray() {
      let result: Vec<i32> =
          from_slice(&[147, 1, 2, 206, 0, 8, 82, 65]).unwrap();
      assert_eq!(vec![1, 2, 545345], result);
  }

  #[test]
  fn test_array_16() {
      let result: Vec<i32> = from_slice(&[
          220, 0, 36, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
          17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33,
          34, 35, 36,
      ])
      .unwrap();
      assert_eq!(
          vec![
              1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
              19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34,
              35, 36,
          ],
          result
      );
  }

  #[test]
  fn test_read_struct() {
      #[derive(Deserialize, PartialEq, Debug)]
      struct Bar {
          bar: u16,
      }

      #[derive(Deserialize, PartialEq, Debug)]
      struct Foo {
          foo: Vec<Bar>,
      }

      let foo = Foo {
          foo: vec![
              Bar { bar: 2 },
              Bar { bar: 4 },
              Bar { bar: 6 },
              Bar { bar: 8 },
              Bar { bar: 10 },
          ],
      };

      let result: Foo = from_slice(&[
          129, 163, 102, 111, 111, 149, 129, 163, 98, 97, 114, 2, 129, 163,
          98, 97, 114, 4, 129, 163, 98, 97, 114, 6, 129, 163, 98, 97, 114, 8,
          129, 163, 98, 97, 114, 10,
      ])
      .unwrap();
      assert_eq!(foo, result);
  }

  #[test]
  fn test_read_enum_number() {
      #[derive(Deserialize, PartialEq, Debug)]
      enum Foo {
          _First,
          Second,
          _Third,
      }

      let foo = Foo::Second;

      let result: Foo = from_slice(&[1]).unwrap();
      assert_eq!(foo, result);
  }

  #[test]
  fn test_read_enum_string() {
      #[derive(Deserialize, PartialEq, Debug)]
      enum Foo {
          _First,
          Second,
          _Third,
      }

      let foo = Foo::Second;

      let result: Foo = from_slice(&[166, 83, 69, 67, 79, 78, 68]).unwrap();
      assert_eq!(foo, result);
  }

  #[test]
  fn test_bigint() {
      let foo = BigIntWrapper(
          num_bigint::BigInt::from_str(
              "170141183460469231731687303715884105727",
          )
          .unwrap(),
      );

      let result: BigIntWrapper = from_slice(&[
          217, 39, 49, 55, 48, 49, 52, 49, 49, 56, 51, 52, 54, 48, 52, 54,
          57, 50, 51, 49, 55, 51, 49, 54, 56, 55, 51, 48, 51, 55, 49, 53, 56,
          56, 52, 49, 48, 53, 55, 50, 55,
      ])
      .unwrap();
      assert_eq!(foo, result);
  }

  #[test]
  fn test_read_bigint_in_struct() {
      use crate::wrappers::polywrap_bigint;

      #[derive(Deserialize, PartialEq, Debug)]
      struct Foo {
          #[serde(with = "polywrap_bigint")]
          big_int: BigInt,
      }

      let foo = Foo {
          big_int: num_bigint::BigInt::from_str(
              "170141183460469231731687303715884105727",
          )
          .unwrap(),
      };

      let result: Foo = from_slice(&[
          129, 167, 98, 105, 103, 95, 105, 110, 116, 217, 39, 49, 55, 48, 49,
          52, 49, 49, 56, 51, 52, 54, 48, 52, 54, 57, 50, 51, 49, 55, 51, 49,
          54, 56, 55, 51, 48, 51, 55, 49, 53, 56, 56, 52, 49, 48, 53, 55, 50,
          55,
      ])
      .unwrap();
      assert_eq!(foo, result);
  }

  #[test]
  fn test_read_json() {
      use serde_json::Value;
      let foo = JSON(Value::Array(vec![Value::String("bar".to_string())]));

      let result: JSON =
          from_slice(&[167, 91, 34, 98, 97, 114, 34, 93]).unwrap();
      assert_eq!(foo, result);
  }

  #[test]
  fn test_read_json_in_struct() {
      use crate::wrappers::polywrap_json;
      use serde_json::Value;

      #[derive(Deserialize, PartialEq, Debug)]
      struct Foo {
          #[serde(with = "polywrap_json")]
          json: Value,
      }

      let foo = Foo {
          json: Value::Array(vec![Value::String("bar".to_string())]),
      };

      let result: Foo = from_slice(&[
          129, 164, 106, 115, 111, 110, 167, 91, 34, 98, 97, 114, 34, 93,
      ])
      .unwrap();
      assert_eq!(foo, result);
  }
}
