pub mod config;
pub mod cookie;
pub mod endpoint;
pub mod env;
pub mod error;
pub mod headers;
pub mod history;
pub mod pairmap;
pub mod request;
pub mod state;

#[cfg(test)]
mod tests;

pub use error::{Error, Result};

use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use crate::config::ConfigManager;
use crate::endpoint::EndpointManager;
use crate::env::EnvManager;
use crate::history::History;
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
}
