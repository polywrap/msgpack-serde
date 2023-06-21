use crate::{
    error::{get_error_message, Error, Result},
    format::Format,
};
use byteorder::{BigEndian, ReadBytesExt};
use serde::de::{
    self, Deserialize, DeserializeSeed, EnumAccess, IntoDeserializer,
    MapAccess, SeqAccess, VariantAccess, Visitor,
};
use std::io::{Cursor, Read};

pub struct Deserializer {
    buffer: Cursor<Vec<u8>>,
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
    if deserializer.buffer.get_ref().len() == 0 {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

impl<'de> Deserializer {
    fn peek_format(&mut self) -> Result<Format> {
        todo!()
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

    // Look at the input data to decide what Serde data model type to
    // deserialize as. Not all data formats are able to support this operation.
    // Formats that support `deserialize_any` are known as self-describing.
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
        // match self.peek_format()? {
        // 'n' => self.deserialize_unit(visitor),
        // 't' | 'f' => self.deserialize_bool(visitor),
        // '"' => self.deserialize_str(visitor),
        // '0'..='9' => self.deserialize_u64(visitor),
        // '-' => self.deserialize_i64(visitor),
        // '[' => self.deserialize_seq(visitor),
        // '{' => self.deserialize_map(visitor),
        // _ => Err(Error::Syntax),
        // }
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

    // Deserialization of compound types like sequences and maps happens by
    // passing the visitor an "Access" object that gives it the ability to
    // iterate through the data contained in the sequence.
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let arr_len = match Format::get_format(self)? {
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
        }?;

        visitor.visit_seq(ArrayAccess::new(self, arr_len))
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
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
      let map_len = match Format::get_format(self)? {
          Format::FixMap(len) => Ok(len as u32),
          Format::Map16 => Ok(ReadBytesExt::read_u16::<BigEndian>(self)? as u32),
          Format::Map32 => Ok(ReadBytesExt::read_u32::<BigEndian>(self)?),
          Format::Nil => Ok(0),
          err_f => {
              let formatted_err = format!(
                "Property must be of type 'map'. {}",
                get_error_message(err_f)
              );
              Err(Error::ExpectedMap(formatted_err))
          },
      }?;

      visitor.visit_map(ExtMapAccess::new(self, map_len))
    }

    // Structs look just like maps in JSON.
    //
    // Notice the `fields` parameter - a "struct" in the Serde data model means
    // that the `Deserialize` implementation is required to know what the fields
    // are before even looking at the input data. Any key-value pairing in which
    // the fields cannot be known ahead of time is probably a map.
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
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

struct ArrayAccess<'a> {
    deserializer: &'a mut Deserializer,
    elements_in_arr: u32,
}

impl<'a> ArrayAccess<'a> {
    pub fn new(
        deserializer: &'a mut Deserializer,
        elements_in_arr: u32,
    ) -> Self {
        Self {
            deserializer,
            elements_in_arr,
        }
    }
}

impl<'a, 'de> SeqAccess<'de> for ArrayAccess<'a> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        if self.elements_in_arr == 0 {
            return Ok(None);
        }

        self.elements_in_arr -= 1;
        seed.deserialize(&mut *self.deserializer).map(Some)
    }
}

struct ExtMapAccess<'a> {
  deserializer: &'a mut Deserializer,
  entries_in_map: u32,
}

impl<'a> ExtMapAccess<'a> {
  pub fn new(
      deserializer: &'a mut Deserializer,
      entries_in_map: u32,
  ) -> Self {
      Self {
          deserializer,
          entries_in_map,
      }
  }
}

impl<'a, 'de> MapAccess<'de> for ExtMapAccess<'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.entries_in_map == 0 {
            return Ok(None);
        }

        seed.deserialize(&mut *self.deserializer).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        self.entries_in_map -= 1;
        seed.deserialize(&mut *self.deserializer)
    }
}

struct StructAccess<'a> {
  deserializer: &'a mut Deserializer,
  entries_in_map: u32,
}

impl<'a, 'de> MapAccess<'de> for StructAccess<'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.entries_in_map == 0 {
            return Ok(None);
        }

        seed.deserialize(&mut *self.deserializer).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        self.entries_in_map -= 1;
        seed.deserialize(&mut *self.deserializer)
    }
}

struct Enum<'a> {
    de: &'a mut Deserializer,
}

impl<'a> Enum<'a> {
    fn new(de: &'a mut Deserializer) -> Self {
        Enum { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for Enum<'a> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        todo!()
    }
}

impl<'de, 'a> VariantAccess<'de> for Enum<'a> {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        todo!()
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
      todo!()
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
      todo!()
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
      todo!()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {}
