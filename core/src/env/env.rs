use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    headers::Headers,
    pairmap::PairMap,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Variables(pub HashMap<String, String>);

impl Deref for Variables {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Variables {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Variables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in self.iter() {
            writeln!(f, "{key}={value}")?;
        }

        Ok(())
    }
}

impl PairMap<'_> for Variables {
    const NAME: &'static str = "variable";

    fn map(&mut self) -> &mut HashMap<String, String> {
        &mut self.0
    }
}

impl Variables {
    pub fn parse(file_content: &str) -> Result<Self> {
        let mut variables = Variables::default();

        for var in file_content.split('\n').filter(|line| !line.is_empty()) {
            variables.set(var)?;
        }

        Ok(variables)
    }
}

#[derive(Clone, Default)]
pub struct Env {
    pub variables: Variables,
    pub headers: Headers,
}

impl Env {
    pub fn header_set(&mut self, name: String, value: String) -> Result {
        self.headers.0.insert(name, value);
        Ok(())
    }
    pub fn header_rm(&mut self, header: &str) -> Result {
        self.headers.remove(header);
        Ok(())
    }
    pub fn header_get(&self, key: &str) -> Result<String> {
        let value = self
            .headers
            .get(key)
            .ok_or(Error::HeaderNotFound)?
            .to_owned();
        Ok(value)
    }
    pub fn var_set(&mut self, key: String, value: String) -> Result {
        self.variables.0.insert(key, value);

        Ok(())
    }
    pub fn var_get(&self, name: &str) -> Option<String> {
        let v = self
            .variables
            .get(name)
            .map(|inner| inner.to_owned())
            .to_owned();

        v
    }
    pub fn vars(&self) -> Variables {
        let vars = self.variables.clone();

        vars
    }
    pub fn var_rm(&mut self, keys: Vec<String>) -> Result {
        for key in keys {
            self.variables
                .remove(&key)
                .ok_or(Error::RemoveHeader)?;
        }

        Ok(())
    }
}
