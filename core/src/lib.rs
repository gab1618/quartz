pub mod config;
pub mod cookie;
pub mod ctx;
pub mod editor;
pub mod endpoint;
pub mod env;
pub mod history;
pub mod pager;
pub mod pairmap;
pub mod snippet;
pub mod state;
pub mod tree;
pub mod validator;

#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::error::Error;
use std::ffi::OsString;
use std::fmt::Display;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use chrono::Utc;
use endpoint::Endpoint;
use hyper::body::{Bytes, HttpBody};
use hyper::header::{HeaderName, HeaderValue};
use hyper::{Body, Client, Uri};

use crate::config::{Config, ConfigManager};
use crate::cookie::CookieJar;
use crate::editor::Editor;
use crate::history::History;
use crate::pager::Pager;
use crate::pairmap::PairMap;
use crate::tree::Tree;
use crate::{
    ctx::{Ctx, CtxArgs},
    endpoint::{EndpointHandle, EndpointPatch},
    env::Env,
    state::StateField,
};

pub type QuartzResult<T = ()> = Result<T, QuartzError>;

#[derive(Debug)]
pub enum QuartzError {
    Internal,
    Init,
    AlreadyInitialized,
    Setup,
}

impl Display for QuartzError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use QuartzError::*;
        match self {
            Internal => writeln!(f, "internal failure"),
            Init => writeln!(f, "Failed to initialize quartz"),
            AlreadyInitialized => writeln!(f, "Quartz already initialized"),
            Setup => writeln!(f, "Failed to setup"),
        }
    }
}

impl Error for QuartzError {}

pub struct Quartz<E: Editor, P: Pager> {
    pub ctx: Ctx,
    editor: E,
    pager: P,
    path: PathBuf,
    config_path: PathBuf,
}

impl<E: Editor, P: Pager> Quartz<E, P> {
    pub fn from_ctx(ctx: Ctx, config_path: PathBuf, editor: E, pager: P) -> Self {
        Self {
            path: ctx.path().to_path_buf(),
            config_path,
            ctx,
            editor,
            pager,
        }
    }
    pub fn init(path: PathBuf, config_path: PathBuf, editor: E, pager: P) -> QuartzResult<Self> {
        let quartz_dir = path.join(".quartz");

        // TODO: properly propagate these errors for better diagnostics context
        if quartz_dir.exists() {
            return Err(QuartzError::AlreadyInitialized);
        }

        std::fs::create_dir(&quartz_dir).map_err(|_| QuartzError::Init)?;

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

        let curr_ctx = Ctx::new(
            path.clone(),
            config_path.clone(),
            CtxArgs {
                from_handle: None,
                early_apply_environment: false,
            },
        )?;

        Ok(Self {
            ctx: curr_ctx,
            editor,
            config_path,
            pager,
            path,
        })
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn current_env(&self) -> Env {
        self.ctx.require_env()
    }
    pub fn get_env(&self, name: &str) -> Option<Env> {
        let env = Env::parse(self.path.clone(), name).ok();
        env
    }
    pub fn create_env(&self, name: &str) -> QuartzResult {
        let new_env = Env::new(name, self.ctx.path().to_path_buf());

        if new_env.exists() {
            return Err(QuartzError::Internal);
        }
        new_env.write().map_err(|_| QuartzError::Internal)?;

        Ok(())
    }
    pub fn get_envs(&self) -> QuartzResult<Vec<String>> {
        let entries =
            std::fs::read_dir(self.ctx.path().join("env")).map_err(|_| QuartzError::Internal)?;
        let env_names = entries
            .map(|entry| {
                let ok_dir_entry = entry.map_err(|_| QuartzError::Internal)?;
                let filename = ok_dir_entry.file_name();
                let str_filename = filename.to_str().ok_or(QuartzError::Internal)?.to_owned();
                Ok(str_filename)
            })
            .collect::<QuartzResult<Vec<String>>>()?;

        Ok(env_names)
    }
    pub fn switch_env(&self, name: &str) -> QuartzResult<Env> {
        let requested_env =
            Env::parse(self.ctx.path().to_path_buf(), name).map_err(|_| QuartzError::Internal)?;
        if !requested_env.exists() {
            return Err(QuartzError::Internal);
        }
        StateField::Env
            .set(&self.ctx, name)
            .map_err(|_| QuartzError::Internal)?;

        Ok(requested_env)
    }
    pub fn remove_env(&self, name: &str) -> QuartzResult {
        let env = Env::new(name, self.ctx.path().to_path_buf());

        if !env.exists() {
            return Err(QuartzError::Internal);
        }
        let curr_env = self.current_env();
        if env.name == curr_env.name {
            return Err(QuartzError::Internal);
        }

        std::fs::remove_dir_all(env.dir()).map_err(|_| QuartzError::Internal)?;

        Ok(())
    }

    pub fn cp_env(&self, src: &str, dest: &str) -> QuartzResult<Env> {
        let src =
            Env::parse(self.ctx.path().to_path_buf(), src).map_err(|_| QuartzError::Internal)?;
        let mut dest = Env::parse(self.ctx.path().to_path_buf(), dest)
            .unwrap_or(Env::new(dest, self.ctx.path().to_path_buf()));

        for (key, value) in src.variables.iter() {
            dest.variables.insert(key.to_string(), value.to_string());
        }

        for (key, value) in src.headers.iter() {
            dest.headers.insert(key.to_string(), value.to_string());
        }

        if dest.exists() {
            dest.update().map_err(|_| QuartzError::Internal)?;
        } else {
            dest.write().map_err(|_| QuartzError::Internal)?;
        }

        Ok(dest)
    }
    pub fn config(&self) -> &ConfigManager {
        &self.ctx.config
    }
    pub fn handle_create(
        &self,
        handle: &str,
        mut patch: endpoint::EndpointPatch,
    ) -> QuartzResult {
        if handle.is_empty() {
            return Err(QuartzError::Internal);
        }

        let handle = EndpointHandle::from(handle);

        if handle.exists(&self.ctx) {
            return Err(QuartzError::Internal);
        }

        let mut endpoint = Endpoint::from(&mut patch);
        endpoint.set_handle(&self.ctx, &handle);

        handle.write(&self.ctx);
        endpoint.write();

        Ok(())
    }

    pub fn handle_switch(
        &self,
        handle: Option<String>,
        mut patch: EndpointPatch,
        empty: bool,
    ) -> QuartzResult {
        let handle = if let Some(mut handle) = handle {
            if handle == "-" {
                if let Ok(previous_handle) = StateField::PreviousEndpoint.get(&self.ctx) {
                    handle = previous_handle;
                } else {
                    return Err(QuartzError::Internal);
                }
            }

            let handle = EndpointHandle::from(handle);

            if !handle.exists(&self.ctx) {
                return Err(QuartzError::Internal);
            }

            let previous = StateField::Endpoint.get(&self.ctx);
            if StateField::Endpoint
                .set(&self.ctx, &handle.path.join("/"))
                .is_ok()
            {
                if let Ok(prev) = previous {
                    let _ = StateField::PreviousEndpoint.set(&self.ctx, &prev);
                }
            } else {
                return Err(QuartzError::Internal);
            }

            handle
        } else {
            self.ctx.require_handle()
        };

        if empty {
            handle.make_empty(&self.ctx);
        }

        if !patch.has_changes() {
            return Ok(());
        }

        let mut endpoint = handle
            .endpoint(&self.ctx)
            .unwrap_or(Endpoint::new(handle.dir(&self.ctx)));

        endpoint.update(&mut patch);
        endpoint.write();

        Ok(())
    }

    pub fn handle_cp(&self, recursive: bool, src: String, dest: String) -> QuartzResult {
        let src_handle = self.ctx.require_input_handle(&src);
        if !src_handle.exists(&self.ctx) {
            panic!("no such handle: {}", src_handle.handle());
        }

        let mut queue = VecDeque::<EndpointHandle>::new();
        queue.push_back(src_handle);

        while let Some(mut src_handle) = queue.pop_front() {
            let endpoint = src_handle.endpoint(&self.ctx);

            if recursive {
                for child in src_handle.children(&self.ctx) {
                    queue.push_back(child.clone());
                }
            }

            src_handle.replace(&src, &dest);
            src_handle.write(&self.ctx);

            if let Some(mut endpoint) = endpoint {
                endpoint.set_handle(&self.ctx, &src_handle);
                endpoint.write();
            }
        }

        Ok(())
    }

    pub fn handle_rm(&self, recursive: bool, handles: Vec<String>) -> QuartzResult {
        for name in handles {
            let handle = EndpointHandle::from(&name);

            if !handle.exists(&self.ctx) {
                eprintln!("no such handle: {name}");
                continue;
            }

            if !handle.children(&self.ctx).is_empty() && !recursive {
                eprintln!(
                    "{} has child handles. Use -r option to confirm",
                    handle.handle(),
                );
                continue;
            }

            if std::fs::remove_dir_all(handle.dir(&self.ctx)).is_ok() {
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
            if !handle.exists(&self.ctx) {
                eprintln!("no such handle: {arg}");
                continue;
            }

            original_handles.push(handle);
            queue.push_back((arg, EndpointHandle::from(arg)));
        }

        while let Some((src, mut handle)) = queue.pop_front() {
            let mut dest = EndpointHandle::from(dest.handle()); // copy
            for child in handle.children(&self.ctx) {
                queue.push_back((src, child));
            }

            let maybe_endpoint = handle.endpoint(&self.ctx);

            if handles.len() >= 2 {
                dest.path.push(handle.path.last().unwrap().to_string());
            }

            handle.replace(src, &dest.handle());
            handle.write(&self.ctx);

            if let Some(mut endpoint) = maybe_endpoint {
                endpoint.set_handle(&self.ctx, &handle);
                endpoint.write();
            }
        }

        for handle in original_handles {
            let _ = std::fs::remove_dir_all(handle.dir(&self.ctx));
        }

        Ok(())
    }
    pub fn handle_tree(&self, handle: Option<String>) -> Tree<EndpointHandle> {
        let tree_base = handle
            .map(|name| {
                let handle = EndpointHandle::from(name);
                handle.tree(&self.ctx)
            })
            .unwrap_or(EndpointHandle::QUARTZ.tree(&self.ctx));
        tree_base
    }

    pub fn edit_with_extension<F>(
        &self,
        path: &Path,
        extension: Option<&str>,
        validate: F,
    ) -> QuartzResult
    where
        F: FnOnce(&str) -> QuartzResult,
    {
        let mut temp_path = self.ctx.path().join("user").join("EDIT");

        let extension: Option<OsString> = {
            if let Some(extension) = extension {
                Some(OsString::from(extension))
            } else {
                path.extension().map(|extension| extension.to_os_string())
            }
        };

        if let Some(extension) = extension {
            temp_path.set_extension(extension);
        }

        if !path.exists() {
            std::fs::File::create(path).map_err(|_| QuartzError::Internal)?;
        }

        std::fs::copy(path, &temp_path).map_err(|_| QuartzError::Internal)?;

        self.editor.edit(&self, &temp_path)?;

        let content = std::fs::read_to_string(&temp_path).map_err(|_| QuartzError::Internal)?;

        if let Err(_err) = validate(&content) {
            std::fs::remove_file(&temp_path).map_err(|_| QuartzError::Internal)?;
        }

        std::fs::rename(&temp_path, path).map_err(|_| QuartzError::Internal)?;
        Ok(())
    }

    pub fn edit<F>(&self, path: &Path, validate: F) -> QuartzResult
    where
        F: FnOnce(&str) -> QuartzResult,
    {
        self.edit_with_extension::<F>(path, None, validate)
    }

    pub fn edit_config(&self) -> QuartzResult {
        let config_file_path = Config::filepath(&self.config_path);

        self.edit(&config_file_path, validator::toml_as::<Config>)?;

        Ok(())
    }

    pub fn handle_edit(&self) -> QuartzResult {
        let handle = self.ctx.require_handle();

        self.edit(
            &handle.dir(&self.ctx).join("endpoint.toml"),
            validator::toml_as::<Endpoint>,
        )?;

        Ok(())
    }
    pub async fn send(
        &self,
        variables: Vec<String>,
        mut patch: EndpointPatch,
        no_follow: bool,
        cookies: Vec<String>,
        aditional_cookie_jar: Option<PathBuf>,
    ) -> QuartzResult<Bytes> {
        let (handle, mut endpoint) = self.ctx.require_endpoint();
        let mut env = self.ctx.require_env();
        for var in variables {
            env.variables.set(&var)?;
        }

        if !endpoint.headers.contains_key("user-agent") {
            endpoint
                .headers
                .insert("user-agent".to_string(), Ctx::user_agent());
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

        endpoint.update(&mut patch);
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
                        HeaderName::from_str(key).map_err(|_| QuartzError::Internal)?,
                        HeaderValue::from_str(val).map_err(|_| QuartzError::Internal)?,
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
                .map_err(|_| QuartzError::Internal)?;

            entry.message(&res);

            if let Some(cookie_header) = res.headers().get("Set-Cookie") {
                let url = endpoint.full_url().map_err(|_| QuartzError::Internal)?;

                cookie_jar.set(
                    url.host().unwrap(),
                    cookie_header.to_str().map_err(|_| QuartzError::Internal)?,
                );
            }

            if no_follow || !res.status().is_redirection() {
                break;
            }

            if let Some(location) = res.headers().get("Location") {
                let location = location.to_str().map_err(|_| QuartzError::Internal)?;

                if location.starts_with('/') {
                    let url = endpoint.full_url().map_err(|_| QuartzError::Internal)?;
                    // This is awful
                    endpoint.url = Uri::builder()
                        .authority(url.authority().unwrap().as_str())
                        .scheme(url.scheme().unwrap().as_str())
                        .path_and_query(location)
                        .build()
                        .map_err(|_| QuartzError::Internal)?
                        .to_string();
                } else if Uri::from_str(location).is_ok() {
                    endpoint.url = location.to_string();
                }
            };
        }

        match aditional_cookie_jar {
            Some(path) => cookie_jar
                .write_at(&path)
                .map_err(|_| QuartzError::Internal)?,
            None => cookie_jar.write().map_err(|_| QuartzError::Internal)?,
        };

        let mut bytes = Bytes::new();

        while let Some(chunk) = res.data().await {
            if let Ok(chunk) = chunk {
                bytes = [bytes, chunk].concat().into();
            }
        }

        entry.message_raw(String::from_utf8(bytes.to_vec()).map_err(|_| QuartzError::Internal)?);

        let h = self.history()?;
        h.write(entry.build()?)?;

        Ok(bytes)
    }
    pub fn history(&self) -> QuartzResult<History> {
        let h = History::new(self.path.clone())?;

        Ok(h)
    }
    pub fn paginate(&self, content: &[u8]) -> QuartzResult {
        self.pager.paginate(&self, content)
    }
    pub fn body_edit(&self, format: Option<String>) -> QuartzResult {
        const POSSIBLE_EXT: [&str; 3] = ["json", "html", "xml"];
        let handle = self.ctx.require_handle();
        let path = handle.dir(&self.ctx).join("body");

        let format = if format.is_some() {
            format
        } else {
            let endpoint = self.ctx.require_endpoint_from_handle(&handle);

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
            // We cannot validate json for now. If we do so, variable notation will fail because it can
            // generate invalid JSON. For exemple:
            //
            // { "value": {{n}} }
            //
            // n must be a number, so we don't wrap it in quotes. This JSON before variables is
            // invalid. A solution may or may not be done later.
            self.edit_with_extension(&path, Some(&format), validator::infallible)?;
        } else {
            self.edit(&path, validator::infallible)?;
        }

        Ok(())
    }
    pub fn set_body(&self, input: String) {
        let handle = self.ctx.require_handle();

        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(handle.dir(&self.ctx).join("body"))
        {
            let _ = file.write_all(input.as_bytes());
        }
    }
}
