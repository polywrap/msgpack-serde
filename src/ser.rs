use std::io::Write as CursorWrite;

use crate::{
    context::Context,
    error::{Error, Result},
    write::Write,
    write_encoder::WriteEncoder,
};
use serde::ser::{self, Serialize};

pub struct ArraySerializer<'a> {
    array_len: u32,
    array_serializer: Serializer,
    parent_encoder: &'a mut WriteEncoder,
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
        self.parent_encoder.write_array_length(&self.array_len)?;
        self.parent_encoder
            .write(&self.array_serializer.write_encoder.get_buffer())?;
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
        self.parent_encoder.write_array_length(&self.array_len)?;
        self.parent_encoder
            .write(&self.array_serializer.write_encoder.get_buffer())?;
        Ok(())
    }
}

pub struct MapSerializer<'a> {
  map_serializer: Serializer,
  parent_encoder: &'a mut WriteEncoder,
}

impl ser::SerializeMap for MapSerializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize {
          key.serialize(&mut self.map_serializer)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> std::result::Result<(), Self::Error>
    where
        T: Serialize {
          value.serialize(&mut self.map_serializer)
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        let map_buffer = self.map_serializer.write_encoder.get_buffer();
        self.parent_encoder.write_ext_map_len(map_buffer.len())?;
        self.parent_encoder.write_ext_map_type()?;
        self.parent_encoder
            .write(&map_buffer)?;
        Ok(())
    }
}

pub struct StructSerializer<'a> {
  entries: u32,
  struct_serializer: Serializer,
  parent_encoder: &'a mut WriteEncoder,
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
        T: Serialize {
          key.serialize(&mut self.struct_serializer)?;
          value.serialize(&mut self.struct_serializer)?;
          self.entries += 1;

          Ok(())
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
      self.parent_encoder.write_map_length(&self.entries)?;
      self.parent_encoder
          .write(&self.struct_serializer.write_encoder.get_buffer())?;
      Ok(())
  }
}


pub struct Serializer {
    write_encoder: WriteEncoder,
}

impl Default for Serializer {
    fn default() -> Self {
        Self {
            write_encoder: WriteEncoder::new(&[], Context::new()),
        }
    }
}

pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut serializer = Serializer::default();
    value.serialize(&mut serializer)?;
    Ok(serializer.write_encoder.get_buffer())
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = ArraySerializer<'a>;
    // TODO: should tuples be serialized as sequences?. Ex: (u8, bool) = [3, true]?
    type SerializeTuple = ArraySerializer<'a>;
    // TODO: should tuples be serialized as sequences?. Ex: Color(u8, bool) = [3, true]?
    type SerializeTupleStruct = Self;
    // TODO: should tuples be serialized as sequences?. Ex: Color(u8, bool) = [3, true]?
    type SerializeTupleVariant = Self;
    type SerializeMap = MapSerializer<'a>;
    type SerializeStruct = StructSerializer<'a>;
    // TODO: how should we serialize struct variants?
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<()> {
        self.write_encoder.write_bool(&v)?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.write_encoder.write_i8(&v)?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.write_encoder.write_i16(&v)?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.write_encoder.write_i32(&v)?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        self.write_encoder.write_i64(&v)?;
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.write_encoder.write_u8(&v)?;
        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.write_encoder.write_u16(&v)?;
        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.write_encoder.write_u32(&v)?;
        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        self.write_encoder.write_u64(&v)?;
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.write_encoder.write_f32(&v)?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        self.write_encoder.write_f64(&v)?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.write_encoder.write_string(&v.to_string())?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        self.write_encoder.write_string(v)?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        self.write_encoder.write_bytes(v)?;
        Ok(())
    }

    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<()> {
        self.write_encoder.write_nil()?;
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _: &'static str,
    ) -> Result<()> {
        self.write_encoder.write_u32(&_variant_index)?;
        Ok(())
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Note that newtype variant (and all of the other variant serialization
    // methods) refer exclusively to the "externally tagged" enum
    // representation.
    //
    // Serialize this to JSON in externally tagged form as `{ NAME: VALUE }`.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _: &'static str,
        _: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        // TODO: optimize for the case where len is defined

        let array_ser = ArraySerializer {
            array_len: 0,
            array_serializer: Serializer::default(),
            parent_encoder: &mut self.write_encoder,
        };
        Ok(array_ser)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        todo!()
    }

    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        todo!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
      let map_ser = MapSerializer {
          map_serializer: Serializer::default(),
          parent_encoder: &mut self.write_encoder,
      };
      Ok(map_ser)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct> {
        let struct_ser = StructSerializer {
            entries: 0,
            struct_serializer: Serializer::default(),
            parent_encoder: &mut self.write_encoder,
        };

        Ok(struct_ser)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        todo!()
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<()> {
        todo!()
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
      todo!()
    }

    fn end(self) -> Result<()> {
      todo!()
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(
        &mut self,
        _: &'static str,
        _: &T,
    ) -> std::result::Result<(), Self::Error>
    where
        T: Serialize {
        todo!()
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::to_vec;
    use serde_derive::Serialize;

    #[test]
    fn test_struct() {
        #[derive(Serialize)]
        struct Test {
            int: u32,
            seq: Vec<&'static str>,
        }

        let test = Test {
            int: 1,
            seq: vec!["a", "b"],
        };

        println!("{:?}", to_vec(&test));
    }

    #[test]
    fn test_ext_map() {
        #[derive(Serialize)]
        struct Test {
            int: u32,
            map: BTreeMap<String, u32>,
        }

        let mut map = BTreeMap::new();
        map.insert("foo".to_string(), 1);

        let test = Test {
            int: 1,
            map,
        };

        println!("{:?}", to_vec(&test));
    }
}
