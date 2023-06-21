use serde::de::{DeserializeSeed, MapAccess};

use crate::{Deserializer, error::{Result, Error}};

pub struct StructAccess<'a> {
  deserializer: &'a mut Deserializer,
  entries_in_map: u32,
}

impl<'a, 'de> MapAccess<'de> for StructAccess<'a> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: DeserializeSeed<'de>,
    {
        if self.entries_in_map == 0 {
            return Ok(None);
        }

        seed.deserialize(&mut *self.deserializer).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: DeserializeSeed<'de>,
    {
        self.entries_in_map -= 1;
        seed.deserialize(&mut *self.deserializer)
    }
}