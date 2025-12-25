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
use crate::endpoint::error::EndpointError;
use crate::env::error::EnvError;
use crate::error::{Error, Result};
use crate::history::History;
use crate::history::error::HistoryError;
use crate::pairmap::PairMap;
use crate::{
    endpoint::{endpoint::EndpointPatch, handle::EndpointHandle},
    env::EnvRef,
    state::StateField,
};

pub const USER_AGENT: &str = concat!("quartz/", env!("CARGO_PKG_VERSION"));

pub struct Quartz {
    config: ConfigManager,
    path: PathBuf,
}

impl Quartz {
    pub fn new(path: PathBuf, config_path: PathBuf) -> Result<Self> {
        let quartz_path = path.join(".quartz");
        let config = ConfigManager::new(config_path);
        Ok(Self {
            path: quartz_path,
            config,
        })
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
    pub fn handle(&self) -> Option<EndpointHandle<'_>> {
        let curr_endpoint_name = StateField::Endpoint.get(self).ok();

        let parsed =
            curr_endpoint_name.map(|handle_name| EndpointHandle::new(self, handle_name.into()));
        parsed
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn env(&self) -> Result<EnvRef<'_>> {
        let curr_env_name = StateField::Env.get(&self).unwrap_or("default".into());

        let parsed_env = EnvRef::new(self, curr_env_name)?;
        Ok(parsed_env)
    }
    pub fn get_env(&self, name: String) -> Option<EnvRef<'_>> {
        let env = EnvRef::new(self, name).ok();
        env
    }
    pub fn create_env(&self, name: String) -> Result<EnvRef<'_>> {
        let new_env = EnvRef::new(self, name)?;

        new_env.save()?;

        Ok(new_env)
    }
    pub fn get_envs(&self) -> Result<impl Iterator<Item = Result<String>>> {
        let entries = std::fs::read_dir(self.path().join("env")).map_err(EnvError::GetEnvs)?;
        let env_names = entries.map(|entry| {
            let ok_dir_entry = entry.map_err(EnvError::GetEnvs)?;
            let filename = ok_dir_entry.file_name();
            let str_filename = filename.to_str().ok_or(EnvError::ParseEnvName)?.to_owned();
            Ok(str_filename)
        });

        Ok(env_names)
    }
    pub fn switch_env(&self, name: String) -> Result<EnvRef<'_>> {
        let requested_env = EnvRef::new(&self, name)?;
        if !requested_env.exists() {
            return Err(EnvError::NotFound.into());
        }
        StateField::Env.set(self, &requested_env.name)?;

        Ok(requested_env)
    }
    pub fn remove_env(&self, name: String) -> Result {
        let env = EnvRef::new(&self, name)?;

        if !env.exists() {
            return Err(EnvError::NotFound.into());
        }
        let curr_env = self.env()?;
        if env.name == curr_env.name {
            return Err(EnvError::EnvInUse(env.name).into());
        }
        env.clean()?;

        Ok(())
    }

    pub fn cp_env(&self, src: String, dest: String) -> Result<EnvRef<'_>> {
        let src = EnvRef::new(&self, src)?;
        let mut dest = EnvRef::new(&self, dest)?;

        for (key, value) in src.variables.iter() {
            dest.variables.insert(key.to_string(), value.to_string());
        }

        for (key, value) in src.headers.iter() {
            dest.headers.insert(key.to_string(), value.to_string());
        }

        dest.save()?;

        Ok(dest)
    }
    pub fn config(&self) -> &ConfigManager {
        &self.config
    }
    pub fn new_handle(&self, name: &str) -> EndpointHandle<'_> {
        let created = EndpointHandle::new(self, name.into());
        created
    }

    pub fn handle_switch(&self, mut handle: String) -> Result<EndpointHandle<'_>> {
        if handle == "-" {
            let previous_handle = StateField::PreviousEndpoint.get(self)?;
            handle = previous_handle;
        }

        let handle = EndpointHandle::new(self, handle.into());

        if !handle.exists() {
            return Err(EndpointError::HandleNotFound(handle.head()).into());
        }

        let previous = StateField::Endpoint.get(self);
        StateField::Endpoint.set(self, &handle.path.join("/"))?;

        if let Ok(prev) = previous {
            StateField::PreviousEndpoint.set(self, &prev)?;
        }

        Ok(handle)
    }

    pub fn handle_cp(&self, recursive: bool, src: &str, dest: &str) -> Result {
        let src_handle = EndpointHandle::new(self, src.into());
        if !src_handle.exists() {
            return Err(EndpointError::HandleNotFound(src.to_owned()).into());
        }
        let dest_handle = EndpointHandle::new(self, dest.into());
        dest_handle.ensure_dir()?;
        if let Some(endpoint) = src_handle.endpoint().ok() {
            dest_handle.write_endpoint(endpoint)?;
        }

        if recursive {
            for child in src_handle.children()? {
                let child_name = child.handle();
                let mut new_handle = EndpointHandle::new(self, child.path);

                // Replace original prefix with the dest one
                let dest_handle_prefix = dest_handle.path[0].clone();
                let _ = std::mem::replace(&mut new_handle.path[0], dest_handle_prefix);

                self.handle_cp(true, &child_name, &new_handle.handle())?;
            }
        }

        Ok(())
    }

    pub fn handle_mv(&self, src: &str, dest: &str) -> Result {
        let src_handle = EndpointHandle::new(self, src.into());
        if !src_handle.exists() {
            return Err(EndpointError::HandleNotFound(src.to_owned()).into());
        }

        // TODO: this might be one of the lazyest solutions so far
        self.handle_cp(true, src, dest)?;
        src_handle.delete(true)?;

        Ok(())
    }

    pub async fn send(
        &self,
        variables: Vec<String>,
        mut patch: EndpointPatch,
        no_follow: bool,
        cookies: Vec<String>,
        aditional_cookie_jar: Option<PathBuf>,
    ) -> Result<Bytes> {
        let handle = self.handle().ok_or(EndpointError::NoHandleInUse)?;
        let curr_env = self.env()?;
        let mut endpoint = handle.endpoint()?;
        endpoint.update(&mut patch)?;
        let resolved_endpoint = endpoint.as_resolved(&handle, &curr_env)?;

        let mut env = self.env()?;
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

        let h = self.history()?;
        h.write(entry.build()?)?;

        Ok(bytes)
    }
    pub fn history(&self) -> Result<History<'_>> {
        let h = History::new(&self.path)?;

        Ok(h)
    }
}
