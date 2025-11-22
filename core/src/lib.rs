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
pub mod tree;

#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use chrono::Utc;
use endpoint::Endpoint;
use hyper::body::{Bytes, HttpBody};
use hyper::header::{HeaderName, HeaderValue};
use hyper::{Body, Client, Uri};

use crate::config::ConfigManager;
use crate::cookie::CookieJar;
use crate::endpoint::error::EndpointError;
use crate::env::error::EnvError;
use crate::error::{QuartzError, QuartzResult};
use crate::history::History;
use crate::history::error::HistoryError;
use crate::pairmap::PairMap;
use crate::tree::Tree;
use crate::{
    endpoint::{EndpointHandle, EndpointPatch},
    env::Env,
    state::StateField,
};

pub const USER_AGENT: &str = concat!("quartz/", env!("CARGO_PKG_VERSION"));

pub struct Quartz {
    config: ConfigManager,
    path: PathBuf,
}

impl Quartz {
    pub fn new(path: PathBuf, config_path: PathBuf) -> QuartzResult<Self> {
        let quartz_path = path.join(".quartz");
        let config = ConfigManager::new(config_path);
        Ok(Self {
            path: quartz_path,
            config,
        })
    }
    pub fn init(path: PathBuf, config_path: PathBuf) -> QuartzResult<Self> {
        let quartz_dir = path.join(".quartz");
        let config = ConfigManager::new(config_path);

        // TODO: properly propagate these errors for better diagnostics context
        if quartz_dir.exists() {
            return Err(QuartzError::AlreadyInitialized);
        }

        std::fs::create_dir(&quartz_dir).map_err(QuartzError::Init)?;

        let ensure_dirs = vec![
            "endpoints",
            "user",
            "user/history",
            "user/state",
            "env",
            "env/default",
        ];

        for dir in ensure_dirs {
            std::fs::create_dir(
                quartz_dir.join(PathBuf::from_str(dir).map_err(|_| QuartzError::Setup)?),
            )
            .map_err(|_| QuartzError::Setup)?;
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
    pub fn make_handle_empty(&self) -> QuartzResult {
        let handle = self.handle().ok_or(EndpointError::NoHandleInUse)?;
        handle.make_empty(&self);

        Ok(())
    }
    pub fn handle(&self) -> Option<EndpointHandle> {
        let curr_endpoint_name = StateField::Endpoint.get(self).ok();

        let parsed = curr_endpoint_name.map(|handle_name| EndpointHandle::from(handle_name));
        parsed
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn env(&self) -> QuartzResult<Env> {
        let curr_env_name = StateField::Env.get(&self).unwrap_or("default".into());

        let parsed_env = Env::parse(self.path().to_path_buf(), &curr_env_name)?;
        Ok(parsed_env)
    }
    pub fn get_env(&self, name: &str) -> Option<Env> {
        let env = Env::parse(self.path.clone(), name).ok();
        env
    }
    pub fn create_env(&self, name: &str) -> QuartzResult {
        let new_env = Env::new(name, self.path().to_path_buf());

        if new_env.exists() {
            return Err(EnvError::AlreadyExistingEnv.into());
        }
        new_env.write()?;

        Ok(())
    }
    pub fn get_envs(&self) -> QuartzResult<Vec<String>> {
        let entries = std::fs::read_dir(self.path().join("env")).map_err(EnvError::GetEnvs)?;
        let env_names = entries
            .map(|entry| {
                let ok_dir_entry = entry.map_err(EnvError::GetEnvs)?;
                let filename = ok_dir_entry.file_name();
                let str_filename = filename.to_str().ok_or(EnvError::ParseEnvName)?.to_owned();
                Ok(str_filename)
            })
            .collect::<QuartzResult<Vec<String>>>()?;

        Ok(env_names)
    }
    pub fn switch_env(&self, name: &str) -> QuartzResult<Env> {
        let requested_env = Env::parse(self.path().to_path_buf(), name)?;
        StateField::Env.set(self, name)?;

        Ok(requested_env)
    }
    pub fn remove_env(&self, name: &str) -> QuartzResult {
        let env = Env::new(name, self.path().to_path_buf());

        if !env.exists() {
            return Err(EnvError::NotFound.into());
        }
        let curr_env = self.env()?;
        if env.name == curr_env.name {
            return Err(EnvError::EnvInUse(env.name).into());
        }

        std::fs::remove_dir_all(env.dir()).map_err(EnvError::DeleteEnv)?;

        Ok(())
    }

    pub fn cp_env(&self, src: &str, dest: &str) -> QuartzResult<Env> {
        let src = Env::parse(self.path().to_path_buf(), src)?;
        let mut dest = Env::parse(self.path().to_path_buf(), dest)
            .unwrap_or(Env::new(dest, self.path().to_path_buf()));

        for (key, value) in src.variables.iter() {
            dest.variables.insert(key.to_string(), value.to_string());
        }

        for (key, value) in src.headers.iter() {
            dest.headers.insert(key.to_string(), value.to_string());
        }

        if dest.exists() {
            dest.update()?;
        } else {
            dest.write()?;
        }

        Ok(dest)
    }
    pub fn config(&self) -> &ConfigManager {
        &self.config
    }
    pub fn handle_create(&self, handle: &str) -> QuartzResult<Endpoint> {
        if handle.is_empty() {
            return Err(EndpointError::EmptyHandle.into());
        }

        let handle = EndpointHandle::from(handle);

        if handle.exists(&self) {
            return Err(EndpointError::AlreadyExistingHandle.into());
        }

        let mut endpoint = Endpoint::default();
        endpoint.set_handle(&self, &handle);

        handle.write(&self)?;
        endpoint.write()?;

        Ok(endpoint)
    }

    pub fn handle_switch(&self, mut handle: String) -> QuartzResult<EndpointHandle> {
        if handle == "-" {
            let previous_handle = StateField::PreviousEndpoint.get(self)?;
            handle = previous_handle;
        }

        let handle = EndpointHandle::from(handle);

        if !handle.exists(&self) {
            return Err(EndpointError::HandleNotFound(handle.head()).into());
        }

        let previous = StateField::Endpoint.get(self);
        StateField::Endpoint.set(self, &handle.path.join("/"))?;

        if let Ok(prev) = previous {
            StateField::PreviousEndpoint.set(self, &prev)?;
        }

        Ok(handle)
    }

    pub fn apply_endpoint_patch(
        &self,
        mut endpoint: Endpoint,
        mut patch: EndpointPatch,
    ) -> QuartzResult {
        endpoint.update(&mut patch)?;
        endpoint.write()?;

        Ok(())
    }

    pub fn handle_cp(&self, recursive: bool, src: String, dest: String) -> QuartzResult {
        let src_handle = EndpointHandle::from(&src);
        if !src_handle.exists(&self) {
            panic!("no such handle: {}", src_handle.handle());
        }

        let mut queue = VecDeque::<EndpointHandle>::new();
        queue.push_back(src_handle);

        while let Some(mut src_handle) = queue.pop_front() {
            let endpoint = src_handle.endpoint(&self);

            if recursive {
                for child in src_handle.children(&self)? {
                    queue.push_back(child.clone());
                }
            }

            src_handle.replace(&src, &dest);
            src_handle.write(&self)?;

            if let Some(mut endpoint) = endpoint {
                endpoint.set_handle(&self, &src_handle);
                endpoint.write()?;
            }
        }

        Ok(())
    }

    pub fn handle_rm(&self, recursive: bool, handles: Vec<String>) -> QuartzResult {
        for name in handles {
            let handle = EndpointHandle::from(&name);

            if !handle.exists(&self) {
                eprintln!("no such handle: {name}");
                continue;
            }

            if !handle.children(&self)?.is_empty() && !recursive {
                eprintln!(
                    "{} has child handles. Use -r option to confirm",
                    handle.handle(),
                );
                continue;
            }

            if std::fs::remove_dir_all(handle.dir(&self)).is_ok() {
                println!("Deleted endpoint {}", handle.handle());
            } else {
                eprintln!("failed to delete endpoint {}", handle.handle());
            }
        }

        Ok(())
    }

    pub fn handle_mv(&self, mut handles: Vec<String>) -> QuartzResult {
        if handles.is_empty() {
            panic!("no handles specified");
        }

        if handles.len() == 1 {
            panic!("missing target handle");
        }

        let dest = EndpointHandle::from(handles.pop().unwrap());
        let mut original_handles = Vec::<EndpointHandle>::new();
        let mut queue = VecDeque::<(&str, EndpointHandle)>::new();

        for arg in &handles {
            let handle = EndpointHandle::from(arg);
            if !handle.exists(&self) {
                eprintln!("no such handle: {arg}");
                continue;
            }

            original_handles.push(handle);
            queue.push_back((arg, EndpointHandle::from(arg)));
        }

        while let Some((src, mut handle)) = queue.pop_front() {
            let mut dest = EndpointHandle::from(dest.handle()); // copy
            for child in handle.children(&self)? {
                queue.push_back((src, child));
            }

            let maybe_endpoint = handle.endpoint(&self);

            if handles.len() >= 2 {
                dest.path.push(handle.path.last().unwrap().to_string());
            }

            handle.replace(src, &dest.handle());
            handle.write(&self)?;

            if let Some(mut endpoint) = maybe_endpoint {
                endpoint.set_handle(&self, &handle);
                endpoint.write()?;
            }
        }

        for handle in original_handles {
            let _ = std::fs::remove_dir_all(handle.dir(&self));
        }

        Ok(())
    }
    pub fn handle_tree(&self, handle: Option<String>) -> QuartzResult<Tree<EndpointHandle>> {
        let tree_base = handle
            .map(|name| {
                let handle = EndpointHandle::from(name);
                handle.tree(&self)
            })
            .unwrap_or(EndpointHandle::QUARTZ.tree(&self))
            .map_err(|e| e.into());
        tree_base
    }

    pub fn handle_endpoint_file_path(&self) -> Option<PathBuf> {
        let handle = self.handle();
        let dir = handle.map(|inner| inner.dir(&self).join("endpoint.toml"));
        dir
    }

    pub async fn send(
        &self,
        variables: Vec<String>,
        mut patch: EndpointPatch,
        no_follow: bool,
        cookies: Vec<String>,
        aditional_cookie_jar: Option<PathBuf>,
    ) -> QuartzResult<Bytes> {
        let handle = self.handle().ok_or(EndpointError::NoHandleInUse)?;
        let mut endpoint = handle.endpoint(self).ok_or(EndpointError::EmptyHandle)?;
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

        endpoint.update(&mut patch)?;
        endpoint.apply_env(&env);

        let body = endpoint.body().cloned();

        let mut res: hyper::Response<Body>;

        loop {
            let mut req = endpoint
                // TODO: Find a way around this clone
                .clone()
                .into_request()
                .unwrap_or_else(|_| panic!("malformed request"));
            for (key, val) in env.headers.iter() {
                if !endpoint.headers.contains_key(key) {
                    req.headers_mut().insert(
                        HeaderName::from_str(key).map_err(|_| QuartzError::ParseHeader)?,
                        HeaderValue::from_str(val).map_err(|_| QuartzError::ParseHeader)?,
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
                .map_err(|_| QuartzError::RequestFailure)?;

            entry.message(&res);

            if let Some(cookie_header) = res.headers().get("Set-Cookie") {
                let url = endpoint.full_url().map_err(|_| QuartzError::ParseUrl)?;

                cookie_jar.set(
                    url.host().unwrap(),
                    cookie_header
                        .to_str()
                        .map_err(|_| QuartzError::ParseCookie)?,
                );
            }

            if no_follow || !res.status().is_redirection() {
                break;
            }

            if let Some(location) = res.headers().get("Location") {
                let location = location
                    .to_str()
                    .map_err(|_| QuartzError::ParseLocationHeader)?;

                if location.starts_with('/') {
                    let url = endpoint
                        .full_url()
                        .map_err(|_| QuartzError::ParseLocationHeader)?;
                    // This is awful
                    endpoint.url = Uri::builder()
                        .authority(url.authority().unwrap().as_str())
                        .scheme(url.scheme().unwrap().as_str())
                        .path_and_query(location)
                        .build()
                        .map_err(|_| QuartzError::ParseLocationHeader)?
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
    pub fn history(&self) -> QuartzResult<History> {
        let h = History::new(self.path.clone())?;

        Ok(h)
    }
    pub fn set_body(&self, input: String) -> QuartzResult {
        let handle = self.handle().ok_or(EndpointError::NoHandleInUse)?;

        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(handle.dir(&self).join("body"))
            .map_err(EndpointError::AccessHandleBody)?;

        f.write_all(input.as_bytes())
            .map_err(EndpointError::AccessHandleBody)?;

        Ok(())
    }
    pub fn get_body(&self) -> QuartzResult<String> {
        let handle = self.handle().ok_or(EndpointError::NoHandleInUse)?;
        let mut f = std::fs::OpenOptions::new()
            .read(true)
            .open(handle.dir(&self).join("body"))
            .map_err(EndpointError::AccessHandleBody)?;
        let mut body_content = String::new();
        f.read_to_string(&mut body_content)
            .map_err(EndpointError::AccessHandleBody)?;

        Ok(body_content)
    }
    pub fn body_file_path(&self) -> QuartzResult<PathBuf> {
        const POSSIBLE_EXT: [&str; 3] = ["json", "html", "xml"];
        let handle = self.handle().ok_or(EndpointError::NoHandleInUse)?;
        let mut path = handle.dir(&self).join("body");

        let format = {
            let endpoint = handle.endpoint(&self).ok_or(EndpointError::EmptyHandle)?;

            if let Some(content) = endpoint.headers.get("content-type") {
                let ext = POSSIBLE_EXT.iter().find_map(|ext| {
                    if content.contains(*ext) {
                        Some(ext.to_string())
                    } else {
                        None
                    }
                });

                ext
            } else {
                None
            }
        };

        if let Some(format) = format {
            path.set_extension(format);
        }

        Ok(path)
    }
}
