use crate::{Quartz, Error, Result};
use std::{io::Write, path::PathBuf};

pub enum StateField {
    Endpoint,
    PreviousEndpoint,
    Env,
}

pub struct State {
    pub handle: Option<String>,
    pub previous_handle: Option<String>,
}

impl StateField {
    pub const STATE_DIR: &'static str = "user/state";

    pub fn file_path(&self, quartz: &Quartz) -> PathBuf {
        quartz.path().join(Self::STATE_DIR).join(match self {
            Self::Endpoint => "endpoint",
            Self::Env => "env",
            Self::PreviousEndpoint => "previous-endpoint",
        })
    }

    pub fn get(&self, quartz: &Quartz) -> Result<String> {
        let file_content =
            std::fs::read_to_string(self.file_path(quartz)).map_err(Error::GetState)?;

        Ok(file_content)
    }

    pub fn set(&self, quartz: &Quartz, value: &str) -> Result {
        let mut file = std::fs::OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(self.file_path(quartz))
            .map_err(Error::SetState)?;

        file.write_all(value.as_bytes())
            .map_err(Error::SetState)
    }
}

impl State {
    pub fn get(&self, quartz: &Quartz, field: StateField) -> Result<String> {
        let overwrite = match field {
            StateField::Endpoint => self.handle.clone(),
            _ => None,
        };

        if let Some(overwrite) = overwrite {
            return Ok(overwrite);
        }

        field.get(quartz)
    }
}
