use crate::{
    error::{get_error_message, Error, Result},
    format::Format,
};
use byteorder::{BigEndian, ReadBytesExt};
use serde::de::{self, Deserialize, Visitor};
use std::io::{Cursor, Read};

use super::{array::ArrayAccess, map::ExtMapAccess};

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
    if deserializer.buffer.position() as usize == deserializer.buffer.get_ref().len() {
        Ok(t)
    } else {
        Err(Error::TrailingCharacters)
    }
}

impl<'de> Deserializer {
    fn peek_format(&mut self) -> Result<Format> {
        let position = self.buffer.position();
        let format = Format::get_format(self)?;
        self.buffer.set_position(position);

        Ok(format)
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

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        todo!()
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
        }?;

        visitor.visit_map(ExtMapAccess::new(self, map_len))
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

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::from_slice;

    #[test]
    fn test_read_string() {
        let result: String =
            from_slice(&[165, 72, 101, 108, 108, 111]).unwrap();
        assert_eq!("Hello".to_string(), result);
    }

    #[test]
    fn test_read_array() {
        let result: Vec<i32> = from_slice(
            &[221, 0, 0, 0, 3, 1, 2, 206, 0, 8, 82, 65]
        ).unwrap();
        let input_arr: Vec<i32> = vec![1, 2, 545345];
        assert_eq!(input_arr, result);
    }

    #[test]
    fn test_read_map() {
        let result: BTreeMap<String, Vec<i32>> = from_slice(
            &[
                223, 0, 0, 0, 1, 163, 102, 111, 111, 221, 0, 0, 0, 3, 1, 2,
                206, 0, 8, 82, 65,
            ]
        ).unwrap();
        assert_eq!(result[&"foo".to_string()], vec![1, 2, 545345]);
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
        let result: u32 =
            from_slice(&[206, 255, 255, 255, 255]).unwrap();
        assert_eq!(u32::MAX, result);
    }

    #[test]
    fn test_read_u64() {
        let result: u64 = from_slice(
            &[207, 255, 255, 255, 255, 255, 255, 255, 255]
        ).unwrap();
        assert_eq!(u64::MAX, result);
    }
}
