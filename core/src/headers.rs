use std::{collections::HashMap, fmt::Display, ops::{Deref, DerefMut}};

use serde::{Deserialize, Serialize};

use crate::{error::QuartzResult, pairmap::PairMap};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Headers(pub HashMap<String, String>);

impl Headers {
    pub fn parse(file_content: &str) -> QuartzResult<Self> {
        let mut headers = Headers::default();
        for header in file_content.lines().filter(|line| !line.is_empty()) {
            headers.set(header)?;
        }
        Ok(headers)
    }
}

impl Deref for Headers {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Headers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Headers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in self.iter() {
            writeln!(f, "{key}: {value}")?;
        }

        Ok(())
    }
}

impl PairMap<'_> for Headers {
    const NAME: &'static str = "header";
    const EXPECTED: &'static str = "<key>: [value]";

    fn map(&mut self) -> &mut HashMap<String, String> {
        &mut self.0
    }

    fn pair(input: &str) -> Option<(String, String)> {
        let (key, value) = input.split_once(": ")?;

        Some((key.to_string(), value.to_string()))
    }
}
