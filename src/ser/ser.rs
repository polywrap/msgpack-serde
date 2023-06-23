use std::io::{Cursor, Write};

use crate::{
    error::{Error, Result},
    format::Format,
};
use byteorder::{BigEndian, WriteBytesExt};
use serde::ser::{self, Serialize};

use super::{
    _struct::StructSerializer, array::ArraySerializer, map::MapSerializer,
};

pub struct Serializer {
    buffer: Cursor<Vec<u8>>,
}

impl Serializer {
    pub fn get_buffer(&self) -> Vec<u8> {
        self.buffer.clone().into_inner()
    }

    fn write_positive_fixed_int(
        &mut self,
        value: u8,
    ) -> std::result::Result<(), Error> {
        assert!(value < 128);
        Ok(Format::set_format(self, Format::PositiveFixInt(value))?)
    }

    fn write_negative_fixed_int(
        &mut self,
        value: i8,
    ) -> std::result::Result<(), Error> {
        assert!((-32..=0).contains(&value));
        Ok(Format::set_format(self, Format::NegativeFixInt(value))?)
    }
}

impl Default for Serializer {
    fn default() -> Self {
        Self {
            buffer: Cursor::new(vec![]),
        }
    }
}

pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut serializer = Serializer::default();
    value.serialize(&mut serializer)?;
    Ok(serializer.get_buffer())
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
        let format = if v { Format::True } else { Format::False };
        Format::set_format(self, format)?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(v as i64)
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        if v >= 0 {
            self.serialize_u64(v as u64)?;
        } else if (-(1 << 5)..0).contains(&v) {
            self.write_negative_fixed_int(v as i8)?;
        } else if v <= i8::MAX as i64 && v >= i8::MIN as i64 {
            Format::set_format(self, Format::Int8)?;
            WriteBytesExt::write_i8(self, v as i8)?;
        } else if v <= i16::MAX as i64 && v >= i16::MIN as i64 {
            Format::set_format(self, Format::Int16)?;
            WriteBytesExt::write_i16::<BigEndian>(self, v as i16)?;
        } else if v <= i32::MAX as i64 && v >= i32::MIN as i64 {
            Format::set_format(self, Format::Int32)?;
            WriteBytesExt::write_i32::<BigEndian>(self, v as i32)?;
        } else {
            Format::set_format(self, Format::Int64)?;
            WriteBytesExt::write_i64::<BigEndian>(self, v)?;
        }
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(v as u64)
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        if v < 1 << 7 {
            Ok(self.write_positive_fixed_int(v as u8)?)
        } else if v <= u8::MAX as u64 {
            Format::set_format(self, Format::Uint8)?;
            Ok(WriteBytesExt::write_u8(self, v as u8)?)
        } else if v <= u16::MAX as u64 {
            Format::set_format(self, Format::Uint16)?;
            Ok(WriteBytesExt::write_u16::<BigEndian>(self, v as u16)?)
        } else if v <= u32::MAX as u64 {
            Format::set_format(self, Format::Uint32)?;
            Ok(WriteBytesExt::write_u32::<BigEndian>(self, v as u32)?)
        } else {
            Format::set_format(self, Format::Uint64)?;
            Ok(WriteBytesExt::write_u64::<BigEndian>(self, v)?)
        }
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(v as f64)?;
        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        fn is_exact_f32(num: f64) -> bool {
            let f32_num = num as f32;
            let f64_num = f32_num as f64;
            f64_num == num
        }

        if is_exact_f32(v) {
            Format::set_format(self, Format::Float32)?;
            WriteBytesExt::write_f32::<BigEndian>(self, (v) as f32)?;
        } else {
            Format::set_format(self, Format::Float64)?;
            WriteBytesExt::write_f64::<BigEndian>(self, v)?;
        }
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<()> {
        self.serialize_str(&v.to_string())?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        let length = v.len() as u32;
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

        self.write_all(v.as_bytes())?;
        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        if v.is_empty() {
            return self.serialize_unit();
        }
        let length = v.len() as u32;
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
        Ok(self.write_all(v)?)
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
        Format::set_format(self, Format::Nil)?;
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
        self.serialize_u32(_variant_index)?;
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
        let array_ser = ArraySerializer::new(self);
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
        let map_ser = MapSerializer::new(self);
        Ok(map_ser)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStruct> {
        let struct_ser = StructSerializer::new(self);
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

impl Write for Serializer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buffer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.buffer.flush()
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
        T: Serialize,
    {
        todo!()
    }

    fn end(self) -> std::result::Result<Self::Ok, Self::Error> {
        todo!()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use serde_derive::Serialize;

    use crate::to_vec;
    use std::{collections::BTreeMap, str::FromStr};

    #[derive(Default, Debug)]
    struct Case<T> {
        _name: String,
        input: T,
        want: Vec<u8>,
    }

    impl<T> Case<T> {
        fn new(name: &str, input: T, want: &[u8]) -> Self {
            Self {
                _name: name.to_string(),
                input,
                want: want.to_vec(),
            }
        }
    }

    #[test]
    fn test_write_nil() {
        let result = to_vec(&()).unwrap();
        assert_eq!([192], result.as_slice());
    }

    #[test]
    fn test_write_bool_false() {
        let result = to_vec(&false).unwrap();
        assert_eq!([194], result.as_slice());
    }

    #[test]
    fn test_write_bool_true() {
        let result = to_vec(&true).unwrap();
        assert_eq!([195], result.as_slice());
    }

    #[test]
    fn test_write_u8() {
        let cases = [
            Case::new("zero", 0, &[0]),
            Case::new("positive fixed int", 1, &[1]),
            Case::new("positive fixed int", 127, &[127]),
            Case::new("8-bit unsigned int", 200, &[204, 200]),
            Case::new("8-bit unsigned int", 255, &[204, 255]),
        ];
        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn test_write_u16() {
        let cases = [
            Case::new("16-bit unsigned int", 256, &[205, 1, 0]),
            Case::new("16-bit unsigned int", 32767, &[205, 127, 255]),
            Case::new("16-bit unsigned int", 32768, &[205, 128, 0]),
            Case::new("16-bit unsigned int", 65535, &[205, 255, 255]),
        ];
        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn test_write_u32() {
        let cases = [
            Case::new("32-bit unsigned int", 65536, &[206, 0, 1, 0, 0]),
            Case::new("32-bit unsigned int", 123456, &[206, 0, 1, 226, 64]),
            Case::new("32-bit unsigned int", 2147483648, &[206, 128, 0, 0, 0]),
            Case::new(
                "32-bit unsigned int",
                4294967295_u32,
                &[206, 255, 255, 255, 255],
            ),
        ];
        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn test_write_i8() {
        let cases = [
            Case::new("zero", 0, &[0]),
            Case::new("negative fixed int", -1, &[255]),
            Case::new("negative fixed int", -31, &[225]),
            Case::new("negative fixed int", -32, &[224]),
            // Case::new("positive fixed int", 1, &[1]),
            // Case::new("positive fixed int", 127, &[127]),
            Case::new("8-bit signed int", -128, &[208, 128]),
            Case::new("8-bit signed int", -100, &[208, 156]),
            Case::new("8-bit signed int", -33, &[208, 223]),
        ];
        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn test_write_i16() {
        let cases = [
            Case::new("16-bit signed int (negative)", -32768, &[209, 128, 0]),
            Case::new("16-bit signed int (negative)", -32767, &[209, 128, 1]),
            Case::new("16-bit signed int (negative)", -3262, &[209, 243, 66]),
            Case::new("16-bit signed int (negative)", -129, &[209, 255, 127]),
            // Case::new("16-bit signed int (positive)", 128, &[209, 0, 128]),
            // Case::new("16-bit signed int (positive)", 32767, &[209, 127, 255]),
        ];
        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn test_write_i32() {
        let cases = [
            Case::new(
                "32-bit signed int (negative)",
                -32769,
                &[210, 255, 255, 127, 255],
            ),
            Case::new(
                "32-bit signed int (negative)",
                -2147483648,
                &[210, 128, 0, 0, 0],
            ),
            Case::new(
                "32-bit signed int (negative)",
                -2147483647,
                &[210, 128, 0, 0, 1],
            ),
            // Case::new("32-bit signed int (positive)", 32768, &[210, 0, 0, 128, 0]),
            // Case::new(
            //     "32-bit signed int (positive)",
            //     123456,
            //     &[210, 0, 1, 226, 64],
            // ),
            // Case::new(
            //     "32-bit signed int (positive)",
            //     2147483647,
            //     &[210, 127, 255, 255, 255],
            // ),
        ];
        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn write_u64() {
        let cases = [Case::new(
            "64-bit unsigned int",
            u64::MAX,
            &[207, 255, 255, 255, 255, 255, 255, 255, 255],
        )];
        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn write_i64() {
        let cases = [
            // Case::new(
            //     "64-bit signed int",
            //     i64::MAX,
            //     &[211, 127, 255, 255, 255, 255, 255, 255, 255],
            // ),
            Case::new(
                "64-bit signed int",
                i64::MIN,
                &[211, 128, 0, 0, 0, 0, 0, 0, 0],
            ),
        ];
        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn test_write_f32() {
        let cases = [Case::new("32-bit float", 0.5, &[202, 63, 0, 0, 0])];

        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn test_write_f64() {
        let cases = [Case::new(
            "64-bit float",
            3.141592653589793,
            &[203, 64, 9, 33, 251, 84, 68, 45, 24],
        )];

        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn test_write_string() {
        let cases = [
          Case::new("Empty String", "", &[160]),
          Case::new("5-char String", "hello", &[165, 104, 101, 108, 108, 111]),
          Case::new(
              "11-char String",
              "hello world",
              &[171, 104, 101, 108, 108, 111, 32, 119, 111, 114, 108, 100],
          ),
          Case::new(
              "31-char String",
              "-This string contains 31 chars-",
              &[
                  191, 45, 84, 104, 105, 115, 32, 115, 116, 114, 105, 110, 103, 32, 99, 111, 110,
                  116, 97, 105, 110, 115, 32, 51, 49, 32, 99, 104, 97, 114, 115, 45,
              ],
          ),
          Case::new(
              "255-char String",
          concat!("This is a str 8 string of 255 bytes ",
        "AC53LgxLLOKm0hfsPa1V0nfMjXtnmkEttruCPjc51dtEMLRJIEu1YoRGd9", "oXnM4CxcIiTc9V2DnAidZz22foIzc3kqHBoXgYskevfoJ5RK",
        "Yp52qvoDPufUebLksFl7astBNEnjPVUX2e3O9O6VKeUpB0iiHQXfzOOjTEK6Xy6ks4zAG2M6jCL01flIJlxplRXCV7 sadsadsadsadasdasaaaaa"),
              &[
              217, 255, 84, 104, 105, 115, 32, 105, 115, 32, 97, 32, 115, 116, 114, 32, 56, 32, 115,
              116, 114, 105, 110, 103, 32, 111, 102, 32, 50, 53, 53, 32, 98, 121, 116, 101, 115, 32,
              65, 67, 53, 51, 76, 103, 120, 76, 76, 79, 75, 109, 48, 104, 102, 115, 80, 97, 49, 86,
              48, 110, 102, 77, 106, 88, 116, 110, 109, 107, 69, 116, 116, 114, 117, 67, 80, 106, 99,
              53, 49, 100, 116, 69, 77, 76, 82, 74, 73, 69, 117, 49, 89, 111, 82, 71, 100, 57, 111,
              88, 110, 77, 52, 67, 120, 99, 73, 105, 84, 99, 57, 86, 50, 68, 110, 65, 105, 100, 90,
              122, 50, 50, 102, 111, 73, 122, 99, 51, 107, 113, 72, 66, 111, 88, 103, 89, 115, 107,
              101, 118, 102, 111, 74, 53, 82, 75, 89, 112, 53, 50, 113, 118, 111, 68, 80, 117, 102,
              85, 101, 98, 76, 107, 115, 70, 108, 55, 97, 115, 116, 66, 78, 69, 110, 106, 80, 86, 85,
              88, 50, 101, 51, 79, 57, 79, 54, 86, 75, 101, 85, 112, 66, 48, 105, 105, 72, 81, 88,
              102, 122, 79, 79, 106, 84, 69, 75, 54, 88, 121, 54, 107, 115, 52, 122, 65, 71, 50, 77,
              54, 106, 67, 76, 48, 49, 102, 108, 73, 74, 108, 120, 112, 108, 82, 88, 67, 86, 55, 32,
              115, 97, 100, 115, 97, 100, 115, 97, 100, 115, 97, 100, 97, 115, 100, 97, 115, 97, 97,
              97, 97, 97
          ])
      ];

        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn test_write_bytes() {
        let cases = [Case::new(
            "Bytes",
            serde_bytes::ByteBuf::from([1]),
            &[196, 1, 1],
        )];

        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn test_write_array() {
        let cases = [
            Case::new(
                "fixarray",
                vec![1, 2, 545345],
                &[147, 1, 2, 206, 0, 8, 82, 65],
            ),
            Case::new(
                "array 16",
                vec![
                    1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17,
                    18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32,
                    33, 34, 35, 36,
                ],
                &[
                    220, 0, 36, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14,
                    15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29,
                    30, 31, 32, 33, 34, 35, 36,
                ],
            ),
        ];

        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn test_write_struct() {
        #[derive(Serialize)]
        struct Bar {
            bar: u16,
        }

        #[derive(Serialize)]
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

        let cases = [Case::new(
            "struct",
            foo,
            &[
                129, 163, 102, 111, 111, 149, 129, 163, 98, 97, 114, 2, 129,
                163, 98, 97, 114, 4, 129, 163, 98, 97, 114, 6, 129, 163, 98,
                97, 114, 8, 129, 163, 98, 97, 114, 10,
            ],
        )];

        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn test_write_ext_generic_map() {
        let mut map1: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
        let _ = map1.insert(1, vec![3, 5, 9]);
        let _ = map1.insert(2, vec![1, 4, 7]);
        let mut map2: BTreeMap<i32, Vec<i32>> = BTreeMap::new();
        for i in 0..16 {
            map2.insert(i, vec![i, i + 1, i + 2]);
        }

        let cases = [
            Case::new(
                "map 8",
                map1,
                &[199, 11, 1, 130, 1, 147, 3, 5, 9, 2, 147, 1, 4, 7],
            ),
            Case::new(
                "map 16",
                map2,
                &[
                    199, 83, 1, 222, 0, 16, 0, 147, 0, 1, 2, 1, 147, 1, 2, 3,
                    2, 147, 2, 3, 4, 3, 147, 3, 4, 5, 4, 147, 4, 5, 6, 5, 147,
                    5, 6, 7, 6, 147, 6, 7, 8, 7, 147, 7, 8, 9, 8, 147, 8, 9,
                    10, 9, 147, 9, 10, 11, 10, 147, 10, 11, 12, 11, 147, 11,
                    12, 13, 12, 147, 12, 13, 14, 13, 147, 13, 14, 15, 14, 147,
                    14, 15, 16, 15, 147, 15, 16, 17,
                ],
            ),
        ];

        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn test_write_ext_generic_map_nested() {
        let mut root_map: BTreeMap<String, BTreeMap<String, u8>> =
            BTreeMap::new();
        let mut sub_map: BTreeMap<String, u8> = BTreeMap::new();
        sub_map.insert("Hello".to_string(), 1);
        sub_map.insert("Heyo".to_string(), 50);
        root_map.insert("Nested".to_string(), sub_map);
        let cases = [Case::new(
            "nested maps",
            root_map,
            &[
                199, 25, 1, 129, 166, 78, 101, 115, 116, 101, 100, 199, 14, 1,
                130, 165, 72, 101, 108, 108, 111, 1, 164, 72, 101, 121, 111,
                50,
            ],
        )];

        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn test_write_enum() {
        #[derive(Serialize)]
        enum Foo {
            _FIRST,
            SECOND,
            _THIRD,
        }

        let foo = Foo::SECOND;

        let cases = [Case::new("enums", foo, &[1])];

        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn test_bignumber() {
        let cases = [Case::new(
            "BigNumber",
            crate::BigNumber::from_str("3124124512.598273468017578125")
                .unwrap(),
            &[
                189, 51, 49, 50, 52, 49, 50, 52, 53, 49, 50, 46, 53, 57, 56,
                50, 55, 51, 52, 54, 56, 48, 49, 55, 53, 55, 56, 49, 50, 53,
            ],
        )];

        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn test_bigint() {
        use num_bigint::BigInt;
        use crate::wrappers::polywrap_bigint;

        #[derive(Serialize)]
        struct Foo {
          #[serde(with="polywrap_bigint")]
          big_int: BigInt
        }

        let cases = [Case::new(
          "BigInt",
          Foo { big_int: BigInt::from(170_141_183_460_469_231_731_687_303_715_884_105_727i128) },
          &[129, 167, 98, 105, 103, 95, 105, 110, 116, 217, 39, 49, 55, 48, 49, 52, 49, 49, 56, 51,
          52, 54, 48, 52, 54, 57, 50, 51, 49, 55, 51, 49, 54, 56, 55, 51, 48, 51, 55, 49, 53, 56, 56, 52,
          49, 48, 53, 55, 50, 55],
        )];

        for case in cases {
            let result = to_vec(&case.input).unwrap();
            assert_eq!(case.want, result.as_slice());
        }
    }

    #[test]
    fn test_json() {
      use serde_json::Value;
      use crate::wrappers::polywrap_json;

      #[derive(Serialize)]
      struct Foo {
        #[serde(with="polywrap_json")]
        json: Value
      }

      let cases = [Case::new(
        "JSON",
        Foo { json: Value::Array(vec![Value::String("bar".to_string())]) },
        &[129, 164, 106, 115, 111, 110, 167, 91, 34, 98, 97, 114, 34, 93],
      )];

      for case in cases {
          let result = to_vec(&case.input).unwrap();
          assert_eq!(case.want, result.as_slice());
      }
  }
}
