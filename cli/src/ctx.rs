use std::ffi::OsString;
use std::path::{Path, PathBuf};

use crate::{Quartz, QuartzError, QuartzResult};
use quartz_core::{config::Config, validator};

pub struct Ctx {
    pub quartz: Quartz,
}

impl Ctx {
    pub fn new(quartz: Quartz) -> Self {
        Self {
            quartz,
        }
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
    pub fn edit<F>(&self, path: &Path, editor: String, validate: F) -> QuartzResult
    where
        F: FnOnce(&str) -> QuartzResult,
    {
        self.edit_with_extension::<F>(path, None, editor, validate)
    }

    /// Opens an editor to modified the specified file at `path` with `extension` in a temporary file.
    ///
    /// After the program exits, `validate` function is ran on temporary file before moving it to
    /// the original file, effectively commiting the edits.
    ///
    /// If `validate` returns [`Err`], the temporary file is deleted while original file is preserved as is.
    ///
    /// # Arguments
    ///
    /// * `path` - A path slice to a file
    /// * `extension` - Which extension to create temporary file with
    /// * `validate` - Validator method to ensure the edit can be saved without errors
    pub fn edit_with_extension<F>(
        &self,
        path: &Path,
        extension: Option<&str>,
        editor: String,
        validate: F,
    ) -> QuartzResult
    where
        F: FnOnce(&str) -> QuartzResult,
    {
        let mut temp_path = self.quartz.path().join("user").join("EDIT");

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

    pub fn edit_config(&self, editor: String, filepath: PathBuf) -> QuartzResult {
        self.edit(&filepath, editor, validator::toml_as::<Config>)?;

        Ok(())
    }

    pub fn body_edit(&self, format: Option<String>) -> QuartzResult {
        const POSSIBLE_EXT: [&str; 3] = ["json", "html", "xml"];
        let handle = self.quartz.handle().ok_or(QuartzError::Internal)?;
        let path = handle.dir(&self.quartz).join("body");
        let editor = self.quartz.config().parse().preferences.editor();

        let format = if format.is_some() {
            format
        } else {
            let endpoint = handle.endpoint(&self.quartz).ok_or(QuartzError::Internal)?;

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
            self.edit_with_extension(&path, Some(&format), editor, validator::infallible)?;
        } else {
            self.edit(&path, editor, validator::infallible)?;
        }

        Ok(())
    }
}
