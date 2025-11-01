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

use crate::ctx::{Ctx, CtxArgs};

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
    fn set(&mut self, input: &'a str) {
        let (key, value) = Self::pair(input)
            .unwrap_or_else(|| panic!("malformed {}. Expected {}", Self::NAME, Self::EXPECTED));

        self.map().insert(key, value);
    }
}

pub struct Quartz {
    ctx: Ctx,
}

impl Quartz {
    pub fn init(path: &PathBuf) -> QuartzResult<Self> {
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

        let curr_ctx = Ctx::new(CtxArgs {
            from_handle: None,
            early_apply_environment: false,
        })?;

        Ok(Self { ctx: curr_ctx })
    }
}
