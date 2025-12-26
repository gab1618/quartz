pub mod config;
pub mod cookie;
pub mod endpoint;
pub mod env;
pub mod error;
pub mod headers;
pub mod history;
pub mod pairmap;
pub mod snippet;
pub mod state;

#[cfg(test)]
mod tests;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use chrono::Utc;
use hyper::body::{Bytes, HttpBody};
use hyper::header::{HeaderName, HeaderValue};
use hyper::{Body, Client, Request, Uri};

use crate::config::ConfigManager;
use crate::cookie::CookieJar;
use crate::endpoint::EndpointManager;
use crate::endpoint::endpoint::EndpointPatch;
use crate::endpoint::error::EndpointError;
use crate::env::EnvManager;
use crate::error::{Error, Result};
use crate::history::History;
use crate::history::error::HistoryError;
use crate::pairmap::PairMap;
use crate::state::StateManager;

pub const USER_AGENT: &str = concat!("quartz/", env!("CARGO_PKG_VERSION"));

pub struct Quartz {
    config: ConfigManager,
    path: PathBuf,
}

impl Quartz {
    pub fn new(path: PathBuf, config_path: PathBuf) -> Self {
        let quartz_path = path.join(".quartz");
        let config = ConfigManager::new(config_path);

        Self {
            path: quartz_path,
            config,
        }
    }
    pub fn init(path: PathBuf, config_path: PathBuf) -> Result<Self> {
        let quartz_dir = path.join(".quartz");
        let config = ConfigManager::new(config_path);

        // TODO: properly propagate these errors for better diagnostics context
        if quartz_dir.exists() {
            return Err(Error::AlreadyInitialized);
        }

        std::fs::create_dir(&quartz_dir).map_err(Error::Init)?;

        let ensure_dirs = vec![
            "endpoints",
            "user",
            "user/history",
            "user/state",
            "env",
            "env/default",
        ];

        for dir in ensure_dirs {
            std::fs::create_dir(quartz_dir.join(PathBuf::from_str(dir).map_err(|_| Error::Setup)?))
                .map_err(|_| Error::Setup)?;
        }

        if path.join(".git").exists() {
            if let Ok(mut gitignore) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path.join(".gitignore"))
            {
                let _ =
                    gitignore.write("\n# Quartz\n.quartz/user\n.quartz/env/**/cookies".as_bytes());
            }
        }

        Ok(Self {
            path: quartz_dir,
            config,
        })
    }
    pub fn state(&self) -> StateManager<'_> {
        StateManager::new(&self.path)
    }
    pub fn endpoint(&self) -> EndpointManager<'_> {
        EndpointManager::new(self)
    }
    pub fn env(&self) -> EnvManager<'_> {
        EnvManager::new(self)
    }
    pub fn history(&self) -> History<'_> {
        History::new(&self.path)
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn config(&self) -> &ConfigManager {
        &self.config
    }

    pub async fn send(
        &self,
        variables: Vec<String>,
        mut patch: EndpointPatch,
        no_follow: bool,
        cookies: Vec<String>,
        aditional_cookie_jar: Option<PathBuf>,
    ) -> Result<Bytes> {
        let endpoint = self.endpoint();
        let handle = endpoint.handle().ok_or(EndpointError::NoHandleInUse)?;
        let env = self.env();
        let curr_env = env.env()?;
        let mut endpoint = handle.endpoint()?;
        endpoint.update(&mut patch)?;
        let resolved_endpoint = endpoint.as_resolved(&handle, &curr_env)?;

        let mut env = env.env()?;
        for var in variables {
            env.variables.set(&var)?;
        }

        if !endpoint.headers.contains_key("user-agent") {
            endpoint
                .headers
                .insert("user-agent".to_string(), USER_AGENT.to_owned());
        }

        let mut cookie_jar = env.cookie_jar();

        let extras = cookies.iter().flat_map(|c| {
            if c.contains('=') {
                return vec![c.to_owned()];
            }

            let path = Path::new(c);
            if !path.exists() {
                panic!("no such file: {c}");
            }

            CookieJar::read(path)
                .unwrap()
                .iter()
                .map(|c| format!("{}={}", c.name(), c.value()))
                .collect()
        });

        let cookie_value = cookie_jar
            .iter()
            .map(|c| format!("{}={}", c.name(), c.value()))
            .chain(extras)
            .collect::<Vec<String>>()
            .join("; ");

        if !cookie_value.is_empty() {
            endpoint
                .headers
                .insert(String::from("Cookie"), cookie_value);
        }

        let mut entry = history::Entry::builder();
        entry
            .handle(handle.handle())
            .timestemp(Utc::now().timestamp_micros());

        let body = handle.body().clone();

        let mut res: hyper::Response<Body>;

        loop {
            let mut req: Request<_> = resolved_endpoint
                // TODO: Find a way around this clone
                .clone()
                .try_into()
                .unwrap_or_else(|_| panic!("malformed request"));
            for (key, val) in env.headers.iter() {
                if !endpoint.headers.contains_key(key) {
                    req.headers_mut().insert(
                        HeaderName::from_str(key).map_err(|_| Error::ParseHeader)?,
                        HeaderValue::from_str(val).map_err(|_| Error::ParseHeader)?,
                    );
                }
            }

            entry.message(&req);
            if let Some(ref body) = body {
                entry.message_raw(body.to_owned());
            }

            let client = {
                let https = hyper_tls::HttpsConnector::new();
                Client::builder().build(https)
            };

            res = client
                .request(req)
                .await
                .map_err(|_| Error::RequestFailure)?;

            entry.message(&res);

            if let Some(cookie_header) = res.headers().get("Set-Cookie") {
                let url = endpoint.full_url().map_err(|_| Error::ParseUrl)?;

                cookie_jar.set(
                    url.host().unwrap(),
                    cookie_header.to_str().map_err(|_| Error::ParseCookie)?,
                );
            }

            if no_follow || !res.status().is_redirection() {
                break;
            }

            if let Some(location) = res.headers().get("Location") {
                let location = location.to_str().map_err(|_| Error::ParseLocationHeader)?;

                if location.starts_with('/') {
                    let url = endpoint
                        .full_url()
                        .map_err(|_| Error::ParseLocationHeader)?;
                    // This is awful
                    endpoint.url = Uri::builder()
                        .authority(url.authority().unwrap().as_str())
                        .scheme(url.scheme().unwrap().as_str())
                        .path_and_query(location)
                        .build()
                        .map_err(|_| Error::ParseLocationHeader)?
                        .to_string();
                } else if Uri::from_str(location).is_ok() {
                    endpoint.url = location.to_string();
                }
            };
        }

        match aditional_cookie_jar {
            Some(path) => cookie_jar.write_at(&path)?,

            None => cookie_jar.write()?,
        };

        let mut bytes = Bytes::new();

        while let Some(chunk) = res.data().await {
            if let Ok(chunk) = chunk {
                bytes = [bytes, chunk].concat().into();
            }
        }

        entry.message_raw(String::from_utf8(bytes.to_vec()).map_err(|_| HistoryError::Serialize)?);

        let h = self.history();
        h.write(entry.build()?)?;

        Ok(bytes)
    }
}
