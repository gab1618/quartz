pub mod config;
pub mod cookie;
pub mod ctx;
pub mod endpoint;
pub mod env;
pub mod history;
pub mod snippet;
pub mod state;
pub mod tree;
pub mod validator;

use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;

use endpoint::Endpoint;

pub type QuartzResult<T = (), E = Box<dyn std::error::Error>> = Result<T, E>;

#[derive(Debug)]
pub enum QuartzError {
    Internal,
}

impl Display for QuartzError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuartzError::Internal => writeln!(f, "internal failure"),
        }
    }
}

impl Error for QuartzError {}

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
    fn set(&mut self, input: &'a str) {
        let (key, value) = Self::pair(input)
            .unwrap_or_else(|| panic!("malformed {}. Expected {}", Self::NAME, Self::EXPECTED));

        self.map().insert(key, value);
    }
}
