use serde::de::{DeserializeSeed, Visitor, EnumAccess, VariantAccess};

use crate::{Deserializer, error::{Result, Error}};

pub struct Enum<'a> {
  de: &'a mut Deserializer,
}

impl<'de, 'a> EnumAccess<'de> for Enum<'a> {
  type Error = Error;
  type Variant = Self;

  fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant)>
  where
      V: DeserializeSeed<'de>,
  {
    let variant = seed.deserialize(&mut *self.de)?;
    Ok((variant, self))
  }
}

impl<'de, 'a> VariantAccess<'de> for Enum<'a> {
  type Error = Error;

  fn unit_variant(self) -> Result<()> {
      Ok(())
  }

  fn newtype_variant_seed<T>(self, _: T) -> Result<T::Value>
  where
      T: DeserializeSeed<'de>,
  {
    todo!()
  }

  fn tuple_variant<V>(self, _len: usize, _: V) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
    todo!()
  }

  fn struct_variant<V>(
      self,
      _fields: &'static [&'static str],
      _: V,
  ) -> Result<V::Value>
  where
      V: Visitor<'de>,
  {
    todo!()
  }
}