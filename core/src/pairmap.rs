use std::collections::HashMap;
use std::hash::Hash;

use crate::{QuartzError, QuartzResult};

pub trait PairMap<'a, K = String, V = String>
where
    K: Eq + PartialEq + Hash + From<&'a str>,
    V: From<&'a str>,
{
    const NAME: &'static str = "key-value pair";
    const EXPECTED: &'static str = "<key>=<value>";

    /// Returns HashMap in the implementation struct.
    fn map(&mut self) -> &mut HashMap<K, V>;

    /// Breaks string into (key, value) tuple.
    fn pair(input: &'a str) -> Option<(K, V)> {
        let (key, value) = input.split_once('=')?;
        let value = value.trim_matches('\'').trim_matches('\"');

        Some((key.into(), value.into()))
    }

    /// Inserts key-value pair into map.
    fn set(&mut self, input: &'a str) -> QuartzResult {
        let (key, value) = Self::pair(input).ok_or(QuartzError::KeymapSet)?;

        self.map().insert(key, value);

        Ok(())
    }
}
