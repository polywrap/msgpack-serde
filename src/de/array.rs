use serde::de::{SeqAccess, DeserializeSeed};

use crate::{Deserializer, error::{Result, Error}};

pub struct ArrayAccess<'a> {
  deserializer: &'a mut Deserializer,
  elements_in_arr: u32,
}

impl<'a> ArrayAccess<'a> {
  pub fn new(
      deserializer: &'a mut Deserializer,
      elements_in_arr: u32,
  ) -> Self {
      Self {
          deserializer,
          elements_in_arr,
      }
  }
}

impl<'a, 'de> SeqAccess<'de> for ArrayAccess<'a> {
  type Error = Error;

  fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
  where
      T: DeserializeSeed<'de>,
  {
      if self.elements_in_arr == 0 {
          return Ok(None);
      }

      self.elements_in_arr -= 1;
      seed.deserialize(&mut *self.deserializer).map(Some)
  }
}