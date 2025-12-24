use crate::validator;
use std::path::Path;

use crate::{Quartz, QuartzError, QuartzResult};
use quartz_core::config::Config;

pub struct Ctx {
    pub quartz: Quartz,
}

impl Ctx {
    pub fn new(quartz: Quartz) -> Self {
        Self { quartz }
    }

    /// Opens an editor to modified the specified file at `path` in a temporary file.
    ///
    /// After the program exits, `validate` function is ran on temporary file before moving it to
    /// the original file, effectively commiting the edits.
    ///
    /// If `validate` returns [`Err`], the temporary file is deleted while original file is preserved as is.
    ///
    /// # Arguments
    ///
    /// * `path` - A path slice to a file
    /// * `validate` - Validator method to ensure the edit can be saved without errors
    pub fn edit<F>(&self, path: &Path, validate: F) -> QuartzResult
    where
        F: FnOnce(&str) -> QuartzResult,
    {
        let mut temp_path = self.quartz.path().join("user").join("EDIT");

        let extension = path.extension().map(|extension| extension.to_os_string());

        if let Some(extension) = extension {
            temp_path.set_extension(extension);
        }

        if !path.exists() {
            std::fs::File::create(path).map_err(|_| QuartzError::Internal)?;
        }

        std::fs::copy(path, &temp_path).map_err(|_| QuartzError::Internal)?;
        let editor = self.quartz.config().parse().preferences.editor();

        let _ = std::process::Command::new(&editor)
            .arg(&temp_path)
            .status()
            .unwrap_or_else(|err| {
                panic!("failed to open editor: {}\n\n{}", editor, err);
            });

        let content = std::fs::read_to_string(&temp_path).map_err(|_| QuartzError::Internal)?;

        if let Err(err) = validate(&content) {
            std::fs::remove_file(&temp_path).map_err(|_| QuartzError::Internal)?;
            panic!("{}", err);
        }

        std::fs::rename(&temp_path, path).map_err(|_| QuartzError::Internal)?;
        Ok(())
    }

    pub fn edit_config(&self) -> QuartzResult {
        let config = self.quartz.config();
        let config_path = config.file_path();
        self.edit(&config_path, validator::toml_as::<Config>)?;

        Ok(())
    }

    pub fn body_edit(&self, format: Option<String>) -> QuartzResult {
        let curr_handle = self.quartz.handle().unwrap();
        let mut file_path = curr_handle.body_file_path();

        if let Some(format) = format {
            file_path.set_extension(format);
        }

        self.edit(&file_path, validator::infallible)?;

        Ok(())
    }
}
