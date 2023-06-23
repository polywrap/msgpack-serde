use std::io::Write;

use byteorder::{BigEndian, WriteBytesExt};
use serde::{ser, Serialize};

use crate::{
    error::{Error, Result},
    format::Format,
};

use super::ser::Serializer;

pub struct ArraySerializer<'a> {
    array_len: u32,
    array_serializer: Serializer,
    parent_encoder: &'a mut Serializer,
}

impl<'a> ArraySerializer<'a> {
    pub fn new(serializer: &'a mut Serializer) -> Self {
        Self {
            array_len: 0,
            array_serializer: Serializer::default(),
            parent_encoder: serializer,
        }
    }

    pub fn write_array_length<W: Write>(
        writer: &mut W,
        length: &u32,
    ) -> std::result::Result<(), Error> {
        let length = *length;
        if length < 16 {
            Format::set_format(writer, Format::FixArray(length as u8))?;
        } else if length <= u16::MAX as u32 {
            Format::set_format(writer, Format::Array16)?;
            WriteBytesExt::write_u16::<BigEndian>(writer, length as u16)?;
        } else {
            Format::set_format(writer, Format::Array32)?;
            WriteBytesExt::write_u32::<BigEndian>(writer, length)?;
        }
        Ok(())
    }
}

impl ser::SerializeSeq for ArraySerializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut self.array_serializer)?;
        self.array_len += 1;
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        ArraySerializer::write_array_length(
            self.parent_encoder,
            &self.array_len,
        )?;
        self.parent_encoder
            .write_all(&self.array_serializer.get_buffer())?;
        Ok(())
    }
}

impl ser::SerializeTuple for ArraySerializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(
        &mut self,
        value: &T,
    ) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        value.serialize(&mut self.array_serializer)?;
        self.array_len += 1;
        Ok(())
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        ArraySerializer::write_array_length(
            self.parent_encoder,
            &self.array_len,
        )?;
        self.parent_encoder
            .write_all(&self.array_serializer.get_buffer())?;
        Ok(())
    }
}
