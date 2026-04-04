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

use chrono::Utc;
pub use error::{Error, Result};
use hyper::body::Bytes;

use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

use crate::config::ConfigManager;
use crate::endpoint::error::EndpointError;
use crate::env::EnvManager;
use crate::history::History;
use crate::history::error::HistoryError;
use crate::request::Request;
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

        if path.join(".git").exists()
            && let Ok(mut gitignore) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(path.join(".gitignore"))
        {
            let _ = gitignore.write("\n# Quartz\n.quartz/user\n.quartz/env/**/cookies".as_bytes());
        }

        Ok(Self {
            path: quartz_dir,
            config,
        })
    }
    /// The request object by itself is just some memory-only stateless value, therefore this
    /// method is meant to deal with all the stateful logic of it, such as recording history and
    /// saving the cookies into a file.
    pub async fn send_request(
        &self,
        mut req: Request,
        no_follow: bool,
        aditional_cookie_jar: Option<PathBuf>,
    ) -> crate::Result<Bytes> {
        let curr_handle = self
            .current_endpoint()
            .ok_or(EndpointError::NoHandleInUse)?;
        let curr_body = curr_handle.body();
        let response = req.send(no_follow).await?;

        let mut entry = crate::history::Entry::builder();
        if let Some(ref body) = curr_body {
            entry.message_raw(body.to_owned());
        }
        entry
            .handle(curr_handle.handle())
            .timestamp(Utc::now().timestamp_micros());
        entry.message_raw(
            String::from_utf8(response.to_vec()).map_err(|_| HistoryError::Serialize)?,
        );

        let h = self.history();
        h.write(entry.build()?)?;

        match aditional_cookie_jar {
            Some(path) => req.cookie_jar().write_at(&path)?,

            None => req.cookie_jar().write()?,
        };
        Ok(response)
    }
    pub fn state(&self) -> StateManager<'_> {
        StateManager::new(&self.path)
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
