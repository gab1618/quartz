use std::{io::Write as _, path::PathBuf};

use crate::{
    error::Result,
    state::{error::StateError, field::StateField},
};

pub mod error;
pub mod field;

pub struct StateManager<'a> {
    path: &'a PathBuf,
}

impl<'a> StateManager<'a> {
    pub const STATE_DIR: &'static str = "user/state";
    pub fn new(path: &'a PathBuf) -> Self {
        Self { path }
    }
    pub fn state_file_path(&self, field: StateField) -> PathBuf {
        let file_name: &'static str = field.into();
        self.path.join(Self::STATE_DIR).join(file_name)
    }
    pub fn get(&self, field: StateField) -> Result<String> {
        let file_path = self.state_file_path(field);
        let file_content = std::fs::read_to_string(file_path).map_err(StateError::GetState)?;

        Ok(file_content)
    }

    pub fn set(&self, field: StateField, value: &str) -> Result {
        let file_path = self.state_file_path(field);
        let mut file = std::fs::OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(file_path)
            .map_err(StateError::SetState)?;

        file.write_all(value.as_bytes())
            .map_err(StateError::SetState)?;
        Ok(())
    }
}
