use colored::Colorize;
use hyper::http::uri::InvalidUri;
use hyper::{Body, Request, Uri};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Display;
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

use crate::Quartz;
use crate::endpoint::error::EndpointError;
use crate::endpoint::resolved_endpoint::ResolvedEndpoint;
use crate::env::EnvRef;
use crate::env::env::Variables;
use crate::error::{QuartzError, QuartzResult};
use crate::headers::Headers;
use crate::pairmap::PairMap;
use crate::state::StateField;

pub mod error;
pub mod resolved_endpoint;

#[cfg(test)]
mod tests;

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

#[derive(Clone)]
pub struct EndpointHandlePath(pub Vec<String>);

impl Deref for EndpointHandlePath {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for EndpointHandlePath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for EndpointHandlePath
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        let path: Vec<String> = value
            .as_ref()
            .trim_matches('/')
            .split('/')
            .map(|s| s.to_string())
            .collect();

        Self(path)
    }
}

#[derive(Clone)]
pub struct EndpointHandle<'a> {
    quartz: &'a Quartz,
    /// List of ordered parent names
    pub path: EndpointHandlePath,
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

    /// Variable values applied from a [`Env`]
    #[serde(skip_serializing, skip_deserializing)]
    pub variables: Variables,

    #[serde(skip_serializing, skip_deserializing)]
    pub path: PathBuf,

    #[serde(skip_serializing, skip_deserializing)]
    pub body: Option<String>,
}

#[derive(Debug, clap::Args)]
#[group(multiple = false)]
pub struct ContentTypeGroup {
    /// Use JSON data in request body with the appropriate content-type header
    #[arg(long, value_name = "DATA")]
    pub json: Option<Option<String>>,

    /// Use raw data in request body
    #[arg(long = "data", short = 'd', value_name = "DATA")]
    pub raw: Option<String>,
}

#[derive(Default, Debug, clap::Args)]
pub struct EndpointPatch {
    /// Patch request URL
    #[arg(long)]
    pub url: Option<String>,

    /// Patch HTTP request method
    #[arg(short = 'X', long = "request")]
    pub method: Option<String>,

    /// Add or patch a parameter to the URL query. This argument can be passed multiple times
    #[arg(short, long, value_name = "PARAM")]
    pub query: Vec<String>,

    /// Add or patch a header. This argument can be passed multiple times
    #[arg(short = 'H', long = "header")]
    pub headers: Vec<String>,

    #[command(flatten)]
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

impl<'a> EndpointHandle<'a> {
    pub fn new(quartz: &'a Quartz, path: EndpointHandlePath) -> Self {
        Self { quartz, path }
    }

    pub fn from_state(quartz: &'a Quartz) -> Option<Self> {
        if let Ok(handle) = StateField::Endpoint.get(quartz) {
            if handle.is_empty() {
                return None;
            }

            return Some(EndpointHandle::new(quartz, handle.into()));
        }

        None
    }

    pub fn head(&self) -> String {
        self.path.last().unwrap_or(&String::new()).clone()
    }

    pub fn dir(&self) -> PathBuf {
        let mut result = self.quartz.path().join("endpoints");

        for parent in self.path.iter() {
            let name = Endpoint::name_to_dir(parent);

            result = result.join(name);
        }

        result
    }

    pub fn handle(&self) -> String {
        self.path.join("/")
    }

    pub fn exists(&self) -> bool {
        let path = self.dir();
        path.exists()
    }

    /// Records files to build this endpoint with `parse` methods.
    pub fn write(&self, quartz: &Quartz) -> QuartzResult {
        let mut dir = quartz.path().join("endpoints");
        for entry in self.path.iter() {
            dir = dir.join(Endpoint::name_to_dir(entry));

            std::fs::create_dir_all(&dir).map_err(EndpointError::SaveHandle)?;

            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(dir.join("spec"))
                .map_err(EndpointError::SaveHandle)?;

            file.write_all(entry.as_bytes())
                .map_err(EndpointError::SaveHandle)?;
        }

        std::fs::create_dir_all(self.dir()).map_err(EndpointError::SaveHandle)?;

        Ok(())
    }

    /// Removes endpoint to make it an empty handle
    pub fn make_empty(&self) {
        if self.endpoint().is_some() {
            let _ = std::fs::remove_file(self.dir().join("endpoint.toml"));
            let _ = std::fs::remove_file(self.dir().join("body"));
        }
    }

    pub fn depth(&self) -> usize {
        self.path.len()
    }

    pub fn children(&self) -> QuartzResult<Vec<EndpointHandle<'_>>> {
        let paths = std::fs::read_dir(self.dir()).map_err(EndpointError::GetHandleChildren)?;
        let valid_paths = paths
            .filter(|entry| entry.is_ok())
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_dir());

        let list = valid_paths
            .map(|path| {
                let spec_file_path = path.join("spec");
                let raw_spec_content =
                    std::fs::read(spec_file_path).map_err(EndpointError::GetHandleChildren)?;
                let spec = String::from_utf8(raw_spec_content)
                    .map_err(|_| EndpointError::ParseHandleSpec)?;

                let mut path = self.path.clone();
                path.push(spec);
                Ok(EndpointHandle::new(self.quartz, path))
            })
            .collect::<QuartzResult<Vec<_>>>()?;

        Ok(list)
    }

    #[must_use]
    pub fn endpoint(&self) -> Option<Endpoint> {
        Endpoint::from_dir(&self.dir()).ok()
    }

    pub fn replace(&mut self, from: &str, to: &str) {
        let handle = self.handle().replace(from, to);
        self.path = EndpointHandle::new(self.quartz, handle.into()).path;
    }
}

impl TryFrom<&mut EndpointPatch> for Endpoint {
    type Error = QuartzError;

    fn try_from(value: &mut EndpointPatch) -> Result<Self, Self::Error> {
        let mut endpoint = Self::default();
        endpoint.update(value)?;

        Ok(endpoint)
    }
}

impl Endpoint {
    pub fn new(path: PathBuf) -> Self {
        Self {
            method: String::from("GET"),
            path,
            ..Default::default()
        }
    }

    pub fn name_to_dir(name: &str) -> String {
        name.trim().replace(['/', '\\'], "-")
    }

    pub fn from_dir(dir: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let bytes = std::fs::read(dir.join("endpoint.toml"))?;
        let content = String::from_utf8(bytes)?;

        let mut endpoint: Endpoint = toml::from_str(&content)?;
        endpoint.path = dir.to_path_buf();

        Ok(endpoint)
    }

    pub fn update(&mut self, src: &mut EndpointPatch) -> QuartzResult {
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
            if let Some(maybe_json) = &data.json {
                self.headers
                    .insert("Content-type".into(), "application/json".into());

                if let Some(json) = maybe_json {
                    self.body = Some(json.to_owned());
                }
            } else if let Some(raw) = &data.raw {
                self.body = Some(raw.to_owned());
            }
        }

        Ok(())
    }

    pub fn to_toml(&self) -> QuartzResult<String> {
        let result = toml::to_string(&self).map_err(EndpointError::SerializeEndpoint)?;
        Ok(result)
    }

    fn key_match_str(key: &str) -> String {
        format!("{{{{{}}}}}", key)
    }

    pub fn load_body(&mut self) -> Option<&String> {
        match std::fs::read_to_string(self.path.join("body")) {
            Ok(mut content) => {
                for (key, value) in self.variables.iter() {
                    let key_match = Self::key_match_str(&key);

                    content = content.replace(&key_match, value);
                }

                if content.trim().is_empty() {
                    return None;
                }

                self.body = Some(content.to_owned());
                self.body.as_ref()
            }
            Err(_) => None,
        }
    }

    pub fn body(&mut self) -> Option<&String> {
        if self.body.is_some() {
            self.body.as_ref()
        } else {
            self.load_body()
        }
    }

    pub fn set_handle(&mut self, handle: &EndpointHandle) {
        self.path = handle.dir().to_path_buf();
    }

    pub fn parent(&self) -> Option<Self> {
        let mut path = self.path.clone();

        if path.pop() {
            Self::from_dir(&path).ok()
        } else {
            None
        }
    }

    pub fn resolve_url(&mut self) {
        let resolved = self.resolved_url();
        self.url = resolved;
    }
    /// Inherits parent URL when it starts with "**".
    pub fn resolved_url(&self) -> String {
        let mut full_url = self.url.clone();
        if self.url.starts_with("**") {
            full_url = self
                .parent()
                .map(|p| {
                    let mut parent_resolved = p.resolved_url();
                    if parent_resolved.ends_with('/') {
                        parent_resolved.pop();
                    }
                    self.url.clone().replacen("**", &parent_resolved, 1).clone()
                })
                .unwrap_or(self.url.clone());
        }

        for (key, value) in self.variables.iter() {
            let key_match = Self::key_match_str(&key);
            full_url = full_url.replace(&key_match, value);
        }
        full_url
    }

    pub fn apply_env(&mut self, env: &EnvRef) {
        self.resolve_url();

        for (key, value) in env.variables.iter() {
            let key_match = Self::key_match_str(&key);

            self.url = self.url.replace(&key_match, value);
            self.method = self.method.replace(&key_match, value);

            *self.headers = self
                .headers
                .iter()
                .map(|(h_key, h_value)| {
                    let h_key = &h_key.replace(&key_match, value);
                    let h_value = &h_value.replace(&key_match, value);

                    (h_key.clone(), h_value.clone())
                })
                .collect();

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

        self.variables = env.variables.clone();
    }

    pub fn as_resolved(&self) -> ResolvedEndpoint {
        let mut resolved = ResolvedEndpoint {
            url: self.resolved_url(),
            method: Default::default(),
            headers: Default::default(),
            body: Default::default(),
        };
        for (key, value) in self.variables.iter() {
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

        resolved
    }

    pub fn full_url(&self) -> Result<Uri, InvalidUri> {
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

            return Uri::try_from(scheme);
        }

        result
    }

    /// Returns the a [`Request`] consuming struct.
    pub fn into_request(mut self) -> Result<Request<Body>, hyper::http::Error> {
        let mut builder = hyper::Request::builder().uri(&self.full_url()?);

        if let Ok(method) = hyper::Method::from_bytes(self.method.as_bytes()) {
            builder = builder.method(method);
        }

        for (key, value) in self.headers.iter() {
            builder = builder.header(key, value);
        }

        if let Some(body) = self.body() {
            builder.body(body.to_owned().into())
        } else {
            builder.body(Body::empty())
        }
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

    pub fn write(&mut self) -> QuartzResult {
        let toml_content = self.to_toml()?;

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.path.join("endpoint.toml"))
            .map_err(EndpointError::SaveEndpoint)?;

        file.write_all(toml_content.as_bytes())
            .map_err(EndpointError::SaveEndpoint)?;

        Ok(())
    }
}

impl Default for Endpoint {
    fn default() -> Self {
        Self {
            method: String::from("GET"),
            url: Default::default(),
            headers: Default::default(),
            variables: Default::default(),
            query: Default::default(),
            path: Default::default(),
            body: Default::default(),
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
