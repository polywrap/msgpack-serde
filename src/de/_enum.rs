use serde::de::{DeserializeSeed, Visitor, EnumAccess, VariantAccess};

use crate::{Deserializer, error::{Result, Error}};

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