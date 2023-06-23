use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};
use serde::{ser, Serialize};

use crate::{
    error::{EncodeError, Error},
    format::{ExtensionType, Format},
};

use super::ser::Serializer;

pub struct MapSerializer<'a> {
    map_serializer: Serializer,
    map_entries: u32,
    parent_encoder: &'a mut Serializer,
}

impl<'a> MapSerializer<'a> {
    pub fn new(serializer: &'a mut Serializer) -> Self {
        Self {
            parent_encoder: serializer,
            map_serializer: Serializer::default(),
            map_entries: 0,
        }
    }

    pub fn write_map_length<W: Write>(
        writer: &mut W,
        length: &u32,
    ) -> std::result::Result<(), Error> {
        let length = *length;
        if length < 16 {
            Format::set_format(writer, Format::FixMap(length as u8))?;
        } else if length <= u16::MAX as u32 {
            Format::set_format(writer, Format::Map16)?;
            WriteBytesExt::write_u16::<BigEndian>(writer, length as u16)?;
        } else {
            Format::set_format(writer, Format::Map32)?;
            WriteBytesExt::write_u32::<BigEndian>(writer, length)?;
        }
        Ok(())
    }

    pub fn write_ext_map_type<W: Write>(
        writer: &mut W,
    ) -> Result<(), EncodeError> {
        Ok(WriteBytesExt::write_u8(
            writer,
            ExtensionType::GenericMap.into(),
        )?)
    }

    pub fn write_ext_map_len<W: Write>(
        writer: &mut W,
        length: usize,
    ) -> std::result::Result<(), Error> {
        if length <= u8::MAX as usize {
            Format::set_format(writer, Format::Ext8)?;
            WriteBytesExt::write_u8(writer, length.try_into().unwrap())?;
        } else if length <= u16::MAX as usize {
            Format::set_format(writer, Format::Ext16)?;
            WriteBytesExt::write_u16::<BigEndian>(
                writer,
                length.try_into().unwrap(),
            )?;
        } else {
            Format::set_format(writer, Format::Ext32)?;
            WriteBytesExt::write_u32::<BigEndian>(
                writer,
                length.try_into().unwrap(),
            )?;
        }

        Ok(())
    }
}

impl ser::SerializeMap for MapSerializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(
        &mut self,
        key: &T,
    ) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        key.serialize(&mut self.map_serializer)?;
        self.map_entries += 1;

        Ok(())
    }

    fn serialize_value<T: ?Sized>(
        &mut self,
        value: &T,
    ) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut self.map_serializer)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        let mut aux_map_encoder = Serializer::default();
        MapSerializer::write_map_length(
            &mut aux_map_encoder,
            &self.map_entries,
        )?;

        aux_map_encoder.write_all(&self.map_serializer.get_buffer())?;

        let map_buffer = aux_map_encoder.get_buffer();

        MapSerializer::write_ext_map_len(
            self.parent_encoder,
            map_buffer.len(),
        )?;
        MapSerializer::write_ext_map_type(self.parent_encoder)?;
        self.parent_encoder.write_all(&map_buffer)?;
        Ok(())
    }
}
