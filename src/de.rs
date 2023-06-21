use crate::{
    error::{get_error_message, Error, Result},
    format::Format,
};
use byteorder::{BigEndian, ReadBytesExt};
use serde::de::{
    self, Deserialize, DeserializeSeed, EnumAccess, IntoDeserializer,
    MapAccess, SeqAccess, VariantAccess, Visitor,
};
use std::{io::{Cursor, Read}, fmt::format};

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

    // Float parsing is stupidly hard.
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

    // The `Serializer` implementation on the previous page serialized chars as
    // single-character strings so handle that representation here.
    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // TODO: maybe better implementation
        let str = self.parse_string()?;

        if str.len() == 1 {
          visitor.visit_char(str.chars().last().unwrap())
        } else {
          Err(Error::ExpectedChar(format!("Expected char, found string: '{}'", str)))
        }
    }

    // Refer to the "Understanding deserializer lifetimes" page for information
    // about the three deserialization flavors of strings in Serde.
    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_borrowed_str(&self.parse_string()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // The `Serializer` implementation on the previous page serialized byte
    // arrays as JSON arrays of bytes. Handle that representation here.
    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
    }

    // An absent optional is represented as the JSON `null` and a present
    // optional is represented as just the contained value.
    //
    // As commented in `Serializer` implementation, this is a lossy
    // representation. For example the values `Some(())` and `None` both
    // serialize as just `null`. Unfortunately this is typically what people
    // expect when working with JSON. Other formats are encouraged to behave
    // more intelligently if possible.
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.peek_format()? {
            Format::Nil => {
              Format::get_format(self)?;
              visitor.visit_none()
            }
            _ => {
              visitor.visit_some(self)
            }
        }
    }

    // In Serde, unit means an anonymous value containing no data.
    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
      match Format::get_format(self)? {
        Format::Nil => {
          visitor.visit_unit()
        }
        format => {
          // TODO: better error messaging
          Err(Error::ExpectedNull(format!("Expected null, found format: {}", format)))
        }
    }
    }

    // Unit struct means a named value containing no data.
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
        // Parse the opening bracket of the sequence.
        if self.next_char()? == '[' {
            // Give the visitor access to each element of the sequence.
            let value = visitor.visit_seq(CommaSeparated::new(self))?;
            // Parse the closing bracket of the sequence.
            if self.next_char()? == ']' {
                Ok(value)
            } else {
                Err(Error::ExpectedArrayEnd)
            }
        } else {
            Err(Error::ExpectedArray)
        }
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently.
    //
    // As indicated by the length parameter, the `Deserialize` implementation
    // for a tuple in the Serde data model is required to know the length of the
    // tuple before even looking at the input data.
    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // Tuple structs look just like sequences in JSON.
    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    // Much like `deserialize_seq` but calls the visitors `visit_map` method
    // with a `MapAccess` implementation, rather than the visitor's `visit_seq`
    // method with a `SeqAccess` implementation.
    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        // Parse the opening brace of the map.
        if self.next_char()? == '{' {
            // Give the visitor access to each entry of the map.
            let value = visitor.visit_map(CommaSeparated::new(self))?;
            // Parse the closing brace of the map.
            if self.next_char()? == '}' {
                Ok(value)
            } else {
                Err(Error::ExpectedMapEnd)
            }
        } else {
            Err(Error::ExpectedMap)
        }
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
        if self.peek_char()? == '"' {
            // Visit a unit variant.
            visitor.visit_enum(self.parse_string()?.into_deserializer())
        } else if self.next_char()? == '{' {
            // Visit a newtype variant, tuple variant, or struct variant.
            let value = visitor.visit_enum(Enum::new(self))?;
            // Parse the matching close brace.
            if self.next_char()? == '}' {
                Ok(value)
            } else {
                Err(Error::ExpectedMapEnd)
            }
        } else {
            Err(Error::ExpectedEnum)
        }
    }

    // An identifier in Serde is the type that identifies a field of a struct or
    // the variant of an enum. In JSON, struct fields and enum variants are
    // represented as strings. In other formats they may be represented as
    // numeric indices.
    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    // Like `deserialize_any` but indicates to the `Deserializer` that it makes
    // no difference which `Visitor` method is called because the data is
    // ignored.
    //
    // Some deserializers are able to implement this more efficiently than
    // `deserialize_any`, for example by rapidly skipping over matched
    // delimiters without paying close attention to the data in between.
    //
    // Some formats are not able to implement this at all. Formats that can
    // implement `deserialize_any` and `deserialize_ignored_any` are known as
    // self-describing.
    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

impl Read for Deserializer {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.buffer.read(&mut *buf)
    }
}

// In order to handle commas correctly when deserializing a JSON array or map,
// we need to track whether we are on the first element or past the first
// element.
struct CommaSeparated<'a> {
    de: &'a mut Deserializer,
    first: bool,
}

impl<'a, 'de> CommaSeparated<'a> {
    fn new(de: &'a mut Deserializer) -> Self {
        CommaSeparated { de, first: true }
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a> SeqAccess<'de> for CommaSeparated<'a> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: DeserializeSeed<'de>,
    {
        // Check if there are no more elements.
        if self.de.peek_char()? == ']' {
            return Ok(None);
        }
        // Comma is required before every element except the first.
        if !self.first && self.de.next_char()? != ',' {
            return Err(Error::ExpectedArrayComma);
        }
        self.first = false;
        // Deserialize an array element.
        seed.deserialize(&mut *self.de).map(Some)
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'de, 'a> MapAccess<'de> for CommaSeparated<'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        // Check if there are no more entries.
        if self.de.peek_char()? == '}' {
            return Ok(None);
        }
        // Comma is required before every entry except the first.
        if !self.first && self.de.next_char()? != ',' {
            return Err(Error::ExpectedMapComma);
        }
        self.first = false;
        // Deserialize a map key.
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        // It doesn't make a difference whether the colon is parsed at the end
        // of `next_key_seed` or at the beginning of `next_value_seed`. In this
        // case the code is a bit simpler having it here.
        if self.de.next_char()? != ':' {
            return Err(Error::ExpectedMapColon);
        }
        // Deserialize a map value.
        seed.deserialize(&mut *self.de)
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

// `EnumAccess` is provided to the `Visitor` to give it the ability to determine
// which variant of the enum is supposed to be deserialized.
//
// Note that all enum deserialization methods in Serde refer exclusively to the
// "externally tagged" enum representation.
impl<'de, 'a> EnumAccess<'de> for Enum<'a> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
    where
        V: DeserializeSeed<'de>,
    {
        // The `deserialize_enum` method parsed a `{` character so we are
        // currently inside of a map. The seed will be deserializing itself from
        // the key of the map.
        let val = seed.deserialize(&mut *self.de)?;
        // Parse the colon separating map key from value.
        if self.de.next_char()? == ':' {
            Ok((val, self))
        } else {
            Err(Error::ExpectedMapColon)
        }
    }
}

// `VariantAccess` is provided to the `Visitor` to give it the ability to see
// the content of the single variant that it decided to deserialize.
impl<'de, 'a> VariantAccess<'de> for Enum<'a> {
    type Error = Error;

    // If the `Visitor` expected this variant to be a unit variant, the input
    // should have been the plain string case handled in `deserialize_enum`.
    fn unit_variant(self) -> Result<()> {
        Err(Error::ExpectedString)
    }

    // Newtype variants are represented in JSON as `{ NAME: VALUE }` so
    // deserialize the value here.
    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }` so
    // deserialize the sequence of data here.
    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_seq(self.de, visitor)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }` so
    // deserialize the inner map here.
    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        de::Deserializer::deserialize_map(self.de, visitor)
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {}
