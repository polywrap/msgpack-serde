use crate::{
    context::Context,
    data_view::DataView,
    error::EncodeError,
    format::{ExtensionType, Format},
    write::Write,
    BigInt, BigNumber, JSON,
};
use byteorder::{BigEndian, WriteBytesExt};
use core::hash::Hash;
use std::{collections::BTreeMap, io::Write as StdioWrite};

#[derive(Debug)]
pub struct WriteEncoder {
    pub context: Context,
    pub view: DataView,
}

impl WriteEncoder {
    pub fn new(buf: &[u8], context: Context) -> Self {
        Self {
            context: context.clone(),
            view: DataView::new(buf, context)
                .expect("Error creating new data view"),
        }
    }

    pub fn get_buffer(&self) -> Vec<u8> {
        self.view.get_buffer()
    }

    pub fn write_negative_fixed_int(
        &mut self,
        value: i8,
    ) -> Result<(), EncodeError> {
        // From 0xe0 (0b11100000) taking last 5 bits, to 0xff (0b11111111), taking last 5 bits
        assert!(-32 <= value && value <= 0);
        Format::set_format(self, Format::NegativeFixInt(value))
            .map_err(|e| EncodeError::FormatWriteError(e.to_string()))
    }

    pub fn write_positive_fixed_int(
        &mut self,
        value: u8,
    ) -> Result<(), EncodeError> {
        assert!(value < 128);
        Format::set_format(self, Format::PositiveFixInt(value))
            .map_err(|e| EncodeError::FormatWriteError(e.to_string()))
    }

    /// Encodes a `u64` value into the buffer using the most efficient representation.
    ///
    /// The MessagePack spec requires that the serializer should use
    /// the format which represents the data in the smallest number of bytes.
    #[doc(hidden)]
    pub fn write_u64(&mut self, value: &u64) -> Result<(), EncodeError> {
        let val = *value;
        if val < 1 << 7 {
            Ok(self.write_positive_fixed_int(val as u8)?)
        } else if val <= u8::MAX as u64 {
            Format::set_format(self, Format::Uint8)?;
            Ok(WriteBytesExt::write_u8(self, val as u8)?)
        } else if val <= u16::MAX as u64 {
            Format::set_format(self, Format::Uint16)?;
            Ok(WriteBytesExt::write_u16::<BigEndian>(self, val as u16)?)
        } else if val <= u32::MAX as u64 {
            Format::set_format(self, Format::Uint32)?;
            Ok(WriteBytesExt::write_u32::<BigEndian>(self, val as u32)?)
        } else {
            Format::set_format(self, Format::Uint64)?;
            Ok(WriteBytesExt::write_u64::<BigEndian>(self, val as u64)?)
        }
    }

    /// Encodes an `i64` value into the buffer using the most efficient representation.
    ///
    /// The MessagePack spec requires that the serializer should use
    /// the format which represents the data in the smallest number of bytes, with the exception of
    /// sized/unsized types.
    #[doc(hidden)]
    pub fn write_i64(&mut self, value: &i64) -> Result<(), EncodeError> {
        let val = *value;

        if val >= 0 {
          Ok(self.write_u64(&(val as u64))?)
        } else if val < 0 && val >= -(1 << 5) {
            Ok(self.write_negative_fixed_int(val as i8)?)
        } else if val <= i8::MAX as i64 && val >= i8::MIN as i64 {
            Format::set_format(self, Format::Int8)?;
            Ok(WriteBytesExt::write_i8(self, val as i8)?)
        } else if val <= i16::MAX as i64 && val >= i16::MIN as i64 {
            Format::set_format(self, Format::Int16)?;
            Ok(WriteBytesExt::write_i16::<BigEndian>(self, val as i16)?)
        } else if val <= i32::MAX as i64 && val >= i32::MIN as i64 {
            Format::set_format(self, Format::Int32)?;
            Ok(WriteBytesExt::write_i32::<BigEndian>(self, val as i32)?)
        } else {
            Format::set_format(self, Format::Int64)?;
            Ok(WriteBytesExt::write_i64::<BigEndian>(self, val as i64)?)
        }
    }

    #[doc(hidden)]
    pub fn write_ext_map_len(
        &mut self,
        length: usize,
    ) -> Result<(), EncodeError> {
        if length <= u8::MAX as usize {
            Format::set_format(self, Format::Ext8)?;
            WriteBytesExt::write_u8(self, length.try_into().unwrap())?;
        } else if length <= u16::MAX as usize {
            Format::set_format(self, Format::Ext16)?;
            WriteBytesExt::write_u16::<BigEndian>(
                self,
                length.try_into().unwrap(),
            )?;
        } else {
            Format::set_format(self, Format::Ext32)?;
            WriteBytesExt::write_u32::<BigEndian>(
                self,
                length.try_into().unwrap(),
            )?;
        }

        Ok(())
    }

    pub fn write_ext_map_type(
      &mut self
    ) -> Result<(), EncodeError> {
      Ok(WriteBytesExt::write_u8(self, ExtensionType::GenericMap.to_u8())?)
    }
}

impl StdioWrite for WriteEncoder {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.view.buffer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.view.buffer.flush()
    }
}

impl Write for WriteEncoder {
    fn write_nil(&mut self) -> Result<(), EncodeError> {
        Format::set_format(self, Format::Nil)
            .map_err(|e| EncodeError::NilWriteError(e.to_string()))
    }

    fn write_bool(&mut self, value: &bool) -> Result<(), EncodeError> {
        let format = if *value { Format::True } else { Format::False };
        Format::set_format(self, format)
            .map_err(|e| EncodeError::BooleanWriteError(e.to_string()))
    }

    fn write_i8(&mut self, value: &i8) -> Result<(), EncodeError> {
        self.write_i64(&(*value as i64))
            .map_err(|e| EncodeError::Int8WriteError(e.to_string()))
    }

    fn write_i16(&mut self, value: &i16) -> Result<(), EncodeError> {
        self.write_i64(&(*value as i64))
            .map_err(|e| EncodeError::Int16WriteError(e.to_string()))
    }

    fn write_i32(&mut self, value: &i32) -> Result<(), EncodeError> {
        self.write_i64(&(*value as i64))
            .map_err(|e| EncodeError::Int32WriteError(e.to_string()))
    }

    fn write_u8(&mut self, value: &u8) -> Result<(), EncodeError> {
        self.write_u64(&(*value as u64))
            .map_err(|e| EncodeError::Uint8WriteError(e.to_string()))
    }

    fn write_u16(&mut self, value: &u16) -> Result<(), EncodeError> {
        self.write_u64(&(*value as u64))
            .map_err(|e| EncodeError::Uint16WriteError(e.to_string()))
    }

    fn write_u32(&mut self, value: &u32) -> Result<(), EncodeError> {
        self.write_u64(&(*value as u64))
            .map_err(|e| EncodeError::Uint32WriteError(e.to_string()))
    }

    fn write_f32(&mut self, value: &f32) -> Result<(), EncodeError> {
        Write::write_f64(self, &(*value as f64)).map_err(|e| EncodeError::Float32WriteError(e.to_string()))
    }

    fn write_f64(&mut self, value: &f64) -> Result<(), EncodeError> {
        let val = *value;

        fn is_exact_f32(num: f64) -> bool {
          let f32_num = num as f32;
          let f64_num = f32_num as f64;
          f64_num == num
        }
        
        if is_exact_f32(val) {
          Format::set_format(self, Format::Float32)?;
          WriteBytesExt::write_f32::<BigEndian>(self, (*value) as f32)
            .map_err(|e| EncodeError::Float32WriteError(e.to_string()))?;
        } else {
          Format::set_format(self, Format::Float64)?;
          WriteBytesExt::write_f64::<BigEndian>(self, *value)
              .map_err(|e| EncodeError::Float64WriteError(e.to_string()))?;
        }

        Ok(())
    }

    fn write_string_length(&mut self, length: &u32) -> Result<(), EncodeError> {
        let length = *length;
        if length < 32 {
            Format::set_format(self, Format::FixStr(length as u8))?;
        } else if length <= u8::MAX as u32 {
            Format::set_format(self, Format::Str8)?;
            WriteBytesExt::write_u8(self, length as u8)?;
        } else if length <= u16::MAX as u32 {
            Format::set_format(self, Format::Str16)?;
            WriteBytesExt::write_u16::<BigEndian>(self, length as u16)?;
        } else {
            Format::set_format(self, Format::Str32)?;
            WriteBytesExt::write_u32::<BigEndian>(self, length)?;
        }
        Ok(())
    }

    fn write_string(&mut self, value: &str) -> Result<(), EncodeError> {
        self.write_string_length(&(value.len() as u32))?;
        self.write_all(value.as_bytes())
            .map_err(|e| EncodeError::StrWriteError(e.to_string()))
    }

    fn write_bytes_length(&mut self, length: &u32) -> Result<(), EncodeError> {
        let length = *length;
        if length <= u8::MAX as u32 {
            Format::set_format(self, Format::Bin8)?;
            WriteBytesExt::write_u8(self, length as u8)?;
        } else if length <= u16::MAX as u32 {
            Format::set_format(self, Format::Bin16)?;
            WriteBytesExt::write_u16::<BigEndian>(self, length as u16)?;
        } else {
            Format::set_format(self, Format::Bin32)?;
            WriteBytesExt::write_u32::<BigEndian>(self, length)?;
        }
        Ok(())
    }

    fn write_bytes(&mut self, buf: &[u8]) -> Result<(), EncodeError> {
        if buf.is_empty() {
            return self.write_nil();
        }
        self.write_bytes_length(&(buf.len() as u32))?;
        self.write_all(buf)
            .map_err(|e| EncodeError::BinWriteError(e.to_string()))
    }

    fn write_bigint(&mut self, value: &BigInt) -> Result<(), EncodeError> {
        self.write_string(&value.to_string())
            .map_err(|e| EncodeError::BigIntWriteError(e.to_string()))
    }

    fn write_bignumber(
        &mut self,
        value: &BigNumber,
    ) -> Result<(), EncodeError> {
        self.write_string(&value.to_string())
            .map_err(|e| EncodeError::BigIntWriteError(e.to_string()))
    }

    fn write_json(&mut self, value: &JSON::Value) -> Result<(), EncodeError> {
        let json_str = JSON::to_string(value)?;
        self.write_string(&json_str)
            .map_err(|e| EncodeError::JSONWriteError(e.to_string()))
    }

    fn write_array_length(&mut self, length: &u32) -> Result<(), EncodeError> {
        let length = *length;
        if length < 16 {
            Format::set_format(self, Format::FixArray(length as u8))?;
        } else if length <= u16::MAX as u32 {
            Format::set_format(self, Format::Array16)?;
            WriteBytesExt::write_u16::<BigEndian>(self, length as u16)?;
        } else {
            Format::set_format(self, Format::Array32)?;
            WriteBytesExt::write_u32::<BigEndian>(self, length)?;
        }
        Ok(())
    }

    fn write_array<T: Clone>(
        &mut self,
        array: &[T],
        mut item_writer: impl FnMut(&mut Self, &T) -> Result<(), EncodeError>,
    ) -> Result<(), EncodeError> {
        self.write_array_length(&(array.len() as u32))?;
        for element in array {
            item_writer(self, element)?;
        }
        Ok(())
    }

    fn write_map_length(&mut self, length: &u32) -> Result<(), EncodeError> {
        let length = *length;
        if length < 16 {
            Format::set_format(self, Format::FixMap(length as u8))?;
        } else if length <= u16::MAX as u32 {
            Format::set_format(self, Format::Map16)?;
            WriteBytesExt::write_u16::<BigEndian>(self, length as u16)?;
        } else {
            Format::set_format(self, Format::Map32)?;
            WriteBytesExt::write_u32::<BigEndian>(self, length)?;
        }
        Ok(())
    }

    fn write_map<K, V: Clone>(
        &mut self,
        map: &BTreeMap<K, V>,
        mut key_writer: impl FnMut(&mut Self, &K) -> Result<(), EncodeError>,
        mut val_writer: impl FnMut(&mut Self, &V) -> Result<(), EncodeError>,
    ) -> Result<(), EncodeError>
    where
        K: Clone + Eq + Hash + Ord,
    {
        self.write_map_length(&(map.len() as u32))?;
        let keys: Vec<_> = map.keys().into_iter().collect();
        for key in keys {
            let value = &map[key];
            key_writer(self, key)?;
            val_writer(self, value)?;
        }
        Ok(())
    }

    fn write_ext_generic_map<K, V: Clone>(
        &mut self,
        map: &BTreeMap<K, V>,
        mut key_writer: impl FnMut(&mut Self, &K) -> Result<(), EncodeError>,
        mut val_writer: impl FnMut(&mut Self, &V) -> Result<(), EncodeError>,
    ) -> Result<(), EncodeError>
    where
        K: Clone + Eq + Hash + Ord,
    {
        let mut encoder = WriteEncoder::new(&[], self.context.clone());
        encoder.write_map(map, key_writer, val_writer)?;

        let buf = encoder.get_buffer();
        let bytelength = buf.len();

        // Encode the extension format + bytelength
        self.write_ext_map_len(bytelength);

        // Set the extension type
        self.write_ext_map_type();

        // Copy the map's encoded buffer
        self.view.buffer.write_all(&buf)?;

        Ok(())
    }

    fn context(&mut self) -> &mut Context {
        &mut self.context
    }
}
