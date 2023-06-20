use std::io::Write;

use serde::{ser, Serialize};

use crate::error::Error;

use super::{map::MapSerializer, ser::Serializer};

pub struct StructSerializer<'a> {
    entries: u32,
    struct_serializer: Serializer,
    parent_encoder: &'a mut Serializer,
}

impl<'a> StructSerializer<'a> {
    pub fn new(serializer: &'a mut Serializer) -> Self {
        Self {
            entries: 0,
            struct_serializer: Serializer::default(),
            parent_encoder: serializer,
        }
    }
}

impl ser::SerializeStruct for StructSerializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> std::result::Result<(), Self::Error>
    where
        T: Serialize,
    {
        key.serialize(&mut self.struct_serializer)?;
        value.serialize(&mut self.struct_serializer)?;
        self.entries += 1;

        Ok(())
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        MapSerializer::write_map_length(self.parent_encoder, &self.entries)?;
        self.parent_encoder
            .write(&self.struct_serializer.get_buffer())?;
        Ok(())
    }
}
