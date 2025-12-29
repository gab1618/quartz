use colored::Colorize;
use hyper::Uri;
use std::path::Path;
use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use crate::endpoint::error::EndpointError;
use crate::endpoint::handle::EndpointHandle;
use crate::endpoint::resolved_endpoint::ResolvedEndpoint;
use crate::env::env::Env;
use crate::{error::Error, headers::Headers, pairmap::PairMap};

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Query(pub HashMap<String, String>);

impl Deref for Query {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Query {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Query {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in self.iter() {
            writeln!(f, "{key}={value}")?;
        }

        Ok(())
    }
}

impl PairMap<'_> for Query {
    const NAME: &'static str = "query param";

    fn map(&mut self) -> &mut HashMap<String, String> {
        &mut self.0
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Endpoint {
    pub url: String,

    /// HTTP Request method
    pub method: String,

    /// Query params.
    pub query: Query,

    /// List of (key, value) pairs.
    pub headers: Headers,
}

impl AsRef<Endpoint> for Endpoint {
    fn as_ref(&self) -> &Endpoint {
        self
    }
}

#[derive(Debug)]
pub struct ContentTypeGroup {
    pub json: Option<Option<String>>,
    pub raw: Option<String>,
}

#[derive(Default, Debug)]
pub struct EndpointPatch {
    pub url: Option<String>,
    pub method: Option<String>,
    pub query: Vec<String>,
    pub headers: Vec<String>,
    pub data: Option<ContentTypeGroup>,
}

impl EndpointPatch {
    pub fn has_changes(&self) -> bool {
        self.url.is_some()
            || self.method.is_some()
            || !self.query.is_empty()
            || !self.headers.is_empty()
    }
}

impl TryFrom<&mut EndpointPatch> for Endpoint {
    type Error = Error;

    fn try_from(value: &mut EndpointPatch) -> Result<Self, Self::Error> {
        let mut endpoint = Self::default();
        endpoint.update(value)?;

        Ok(endpoint)
    }
}

impl Endpoint {
    pub fn new() -> Self {
        Self {
            method: String::from("GET"),
            ..Default::default()
        }
    }

    pub fn name_to_dir(name: &str) -> String {
        name.trim().replace(['/', '\\'], "-")
    }

    pub fn from_dir(dir: &Path) -> crate::Result<Self> {
        let bytes =
            std::fs::read(dir.join("endpoint.toml")).map_err(EndpointError::ReadHandleSpec)?;
        let content = String::from_utf8(bytes).map_err(|_| EndpointError::ParseHandleSpec)?;

        let endpoint: Endpoint =
            toml::from_str(&content).map_err(|_| EndpointError::ParseHandleSpec)?;

        Ok(endpoint)
    }

    pub fn update(&mut self, src: &mut EndpointPatch) -> crate::Result {
        if let Some(method) = &mut src.method {
            std::mem::swap(&mut self.method, method);
        }

        if let Some(url) = &mut src.url {
            std::mem::swap(&mut self.url, url);
        }

        for input in &src.query {
            self.query.set(input)?;
        }

        for input in &src.headers {
            self.headers.set(input)?;
        }

        for input in &src.query {
            self.query.set(input)?;
        }

        if let Some(data) = &src.data {
            if let Some(_maybe_json) = &data.json {
                self.headers
                    .insert("Content-type".into(), "application/json".into());
            }
        }

        Ok(())
    }

    pub fn to_toml(&self) -> crate::Result<String> {
        let result = toml::to_string(&self).map_err(EndpointError::SerializeEndpoint)?;
        Ok(result)
    }

    fn key_match_str(key: &str) -> String {
        format!("{{{{{}}}}}", key)
    }

    /// Inherits parent URL when it starts with "**".
    pub fn resolved_url(&self, handle: &EndpointHandle, env: &Env) -> crate::Result<String> {
        let mut full_url = self.url.clone();
        if self.url.starts_with("**") {
            full_url = handle
                .parent()
                .map(|p| {
                    let parent_endpoint = p.endpoint()?;
                    let mut parent_resolved = parent_endpoint.resolved_url(&p, env)?;
                    if parent_resolved.ends_with('/') {
                        parent_resolved.pop();
                    }
                    crate::Result::Ok(self.url.clone().replacen("**", &parent_resolved, 1).clone())
                })
                .unwrap_or(Ok(self.url.clone()))?;
        }

        for (key, value) in env.vars().iter() {
            let key_match = Self::key_match_str(&key);
            full_url = full_url.replace(&key_match, value);
        }
        Ok(full_url)
    }

    pub fn as_resolved(
        &mut self,
        handle: &EndpointHandle,
        env: &Env,
    ) -> crate::Result<ResolvedEndpoint> {
        for (key, value) in env.variables.iter() {
            let key_match = Self::key_match_str(&key);
            *self.query = self
                .query
                .iter()
                .map(|(h_key, h_value)| {
                    let h_key = &h_key.replace(&key_match, value);
                    let h_value = &h_value.replace(&key_match, value);

                    (h_key.clone(), h_value.clone())
                })
                .collect();
        }
        let mut resolved = ResolvedEndpoint {
            url: self.resolved_url(handle, env)?,
            method: Default::default(),
            headers: Default::default(),
            body: Default::default(),
        };
        for (key, value) in env.vars().iter() {
            let key_match = Self::key_match_str(&key);

            resolved.method = self.method.replace(&key_match, value);

            *resolved.headers = self
                .headers
                .iter()
                .map(|(h_key, h_value)| {
                    let h_key = &h_key.replace(&key_match, value);
                    let h_value = &h_value.replace(&key_match, value);

                    (h_key.clone(), h_value.clone())
                })
                .collect();
        }

        Ok(resolved)
    }

    pub fn full_url(&self) -> crate::error::Result<Uri> {
        let query_string = self.query_string();

        let mut url = self.url.clone();

        if !query_string.is_empty() {
            let delimiter = if self.url.contains('?') { '&' } else { '?' };
            url.push(delimiter);
            url.push_str(&query_string);
        }

        let result = Uri::try_from(&url);

        if result.is_err() && !url.contains("://") {
            let mut scheme = "http://".to_owned();
            scheme.push_str(&url);

            return Uri::try_from(scheme).map_err(|_| EndpointError::SerializeUri.into());
        }

        result.map_err(|_| EndpointError::SerializeUri.into())
    }

    pub fn colored_method(&self) -> colored::ColoredString {
        colored_method(&self.method)
    }

    /// Return a query string based off of defined queries.
    ///
    /// ## Example
    ///
    /// A hash map composed of:
    ///
    /// ```toml
    /// [query]
    /// v = 9000
    /// fields = "lorem,ipsum"
    /// ```
    ///
    /// would return: v=9000&fields=lorem,ipsum
    pub fn query_string(&self) -> String {
        let mut result: Vec<String> = Vec::new();

        for (key, value) in self.query.iter() {
            result.push(format!("{key}={value}"));
        }

        result.sort();
        result.join("&")
    }
}

impl Default for Endpoint {
    fn default() -> Self {
        Self {
            method: String::from("GET"),
            url: Default::default(),
            headers: Default::default(),
            query: Default::default(),
        }
    }
}

pub fn colored_method(value: &str) -> colored::ColoredString {
    match value {
        "GET" => value.blue(),
        "POST" => value.green(),
        "PUT" => value.yellow(),
        "PATCH" => value.yellow(),
        "DELETE" => value.red(),
        "OPTIONS" => value.cyan(),
        "HEAD" => value.cyan(),
        "---" => value.dimmed(),
        _ => value.white(),
    }
}
