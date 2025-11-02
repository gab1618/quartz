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

#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::hash::Hash;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use endpoint::Endpoint;

use crate::{
    ctx::{Ctx, CtxArgs},
    env::{Env, Variables},
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
        let (key, value) = Self::pair(input).ok_or(QuartzError::Internal)?;

        self.map().insert(key, value);

        Ok(())
    }
}

pub struct Quartz {
    pub ctx: Ctx,
}

impl Quartz {
    pub fn from_ctx(ctx: Ctx) -> Self {
        Self { ctx }
    }
    pub fn init(path: &PathBuf, config_path: PathBuf) -> QuartzResult<Self> {
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
            config_path,
            CtxArgs {
                from_handle: None,
                early_apply_environment: false,
            },
        )?;

        Ok(Self { ctx: curr_ctx })
    }
    pub fn current_env(&self) -> Env {
        self.ctx.require_env()
    }
    pub fn create_env(&self, name: &str) -> QuartzResult {
        let new_env = Env::new(name);

        if new_env.exists(&self.ctx) {
            return Err(QuartzError::Internal);
        }
        new_env
            .write(&self.ctx)
            .map_err(|_| QuartzError::Internal)?;

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
    pub fn switch_env(&self, name: &str) -> QuartzResult {
        let requested_env = Env::new(name);
        if !requested_env.exists(&self.ctx) {
            return Err(QuartzError::Internal);
        }
        StateField::Env
            .set(&self.ctx, name)
            .map_err(|_| QuartzError::Internal)
    }
    pub fn remove_env(&self, name: &str) -> QuartzResult {
        let env = Env::new(name);

        if !env.exists(&self.ctx) {
            return Err(QuartzError::Internal);
        }
        let curr_env = self.current_env();
        if env.name == curr_env.name {
            return Err(QuartzError::Internal);
        }

        std::fs::remove_dir_all(env.dir(&self.ctx)).map_err(|_| QuartzError::Internal)?;

        Ok(())
    }

    pub fn cp_env(&self, from: &str, to: &str) -> QuartzResult {
        let src = Env::parse(&self.ctx, from).map_err(|_| QuartzError::Internal)?;
        let mut dest = Env::parse(&self.ctx, to).unwrap_or(Env::new(to));

        for (key, value) in src.variables.iter() {
            dest.variables.insert(key.to_string(), value.to_string());
        }

        for (key, value) in src.headers.iter() {
            dest.headers.insert(key.to_string(), value.to_string());
        }

        if dest.exists(&self.ctx) {
            dest.update(&self.ctx).map_err(|_| QuartzError::Internal)?;
        } else {
            dest.write(&self.ctx).map_err(|_| QuartzError::Internal)?;
        }

        Ok(())
    }

    pub fn header_set(&self, header: &str) -> QuartzResult {
        let mut env = self.current_env();
        env.headers.set(header)?;
        env.update(&self.ctx).map_err(|_| QuartzError::Internal)?;
        Ok(())
    }
    pub fn header_rm(&self, header: &str) -> QuartzResult {
        let mut env = self.current_env();
        env.headers.remove(header);
        env.update(&self.ctx).map_err(|_| QuartzError::Internal)?;
        Ok(())
    }
    pub fn header_get(&self, key: &str) -> QuartzResult<String> {
        let env = self.current_env();
        let value = env
            .headers
            .get(key)
            .ok_or(QuartzError::Internal)?
            .to_owned();
        Ok(value)
    }
    pub fn var_set(&self, var: &str) -> QuartzResult {
        let mut curr_env = self.current_env();

        curr_env.variables.set(var)?;
        curr_env.update(&self.ctx)?;

        Ok(())
    }
    pub fn var_get(&self, name: &str) -> Option<String> {
        let curr_env = self.current_env();

        let v = curr_env
            .variables
            .get(name)
            .map(|inner| inner.to_owned())
            .to_owned();

        v
    }
    pub fn vars_get(&self) -> Variables {
        let curr_env = self.current_env();
        let vars = curr_env.variables;

        vars
    }
    pub fn var_rm(&self, keys: Vec<String>) -> QuartzResult {
        let mut env = self.current_env();

        for key in keys {
            env.variables.remove(&key).ok_or(QuartzError::Internal)?;
        }

        env.update(&self.ctx).map_err(|_| QuartzError::Internal)?;
        Ok(())
    }
    pub fn config_set(&self, key: &str, value: &str) -> QuartzResult {
        let mut curr_config = self.ctx.config.parse();
        match key {
            "preferences.editor" => curr_config.preferences.set_editor(value),
            "preferences.pager" => curr_config.preferences.set_pager(value),
            "ui.colors" => curr_config.ui.set_colors(matches!(value, "true")),
            _ => {
                return Err(QuartzError::Internal);
            }
        };

        self.ctx.config.save(curr_config);

        Ok(())
    }
    pub fn config_get(&self, key: &str) -> QuartzResult<String> {
        let value = match key {
            "preferences.editor" => Some(self.ctx.config.parse().preferences.editor()),
            "preferences.pager" => Some(self.ctx.config.parse().preferences.pager()),
            "ui.colors" => Some(self.ctx.config.parse().ui.colors().to_string()),
            _ => None,
        }
        .ok_or(QuartzError::Internal)?;

        Ok(value)
    }
    pub fn config_ls(&self) -> QuartzResult<String> {
        let content = toml::to_string(&self.ctx.config.parse()).map_err(|_| QuartzError::Internal)?;

        Ok(content)
    }
}
