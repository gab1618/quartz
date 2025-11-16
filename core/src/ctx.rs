use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use colored::Colorize;

use crate::config::Config;
use crate::state::State;
use crate::{QuartzError, QuartzResult, validator};

pub struct CtxArgs {
    pub from_handle: Option<String>,
    pub early_apply_environment: bool,
}

pub struct Ctx {
    pub state: State,
    path: PathBuf,
    code: ExitCode,
}

impl Ctx {
    pub fn new(mut dir: PathBuf) -> QuartzResult<Self> {
        let state = State {
            handle: None,
            previous_handle: None,
        };

        loop {
            if dir.join(".quartz").exists() {
                break;
            }

            if !dir.pop() {
                panic!("could not find a quartz project");
            }
        }

        Ok(Ctx {
            state,
            path: dir.join(".quartz"),
            code: ExitCode::default(),
        })
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
        let mut temp_path = self.path().join("user").join("EDIT");

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

    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    pub fn code(&mut self, value: ExitCode) {
        self.code = value;
    }

    pub fn confirm(&self, message: &str) -> bool {
        println!("{} {}", message, "(Y/n)".dimmed());

        std::io::stdout().flush().unwrap();

        let term = console::Term::stdout();
        let ch = term.read_char().unwrap_or('n').to_ascii_lowercase();
        if ch == '\n' {
            return true;
        }

        ch == 'y'
    }

    #[inline]
    #[must_use]
    pub fn exit_code(&self) -> &ExitCode {
        &self.code
    }

    pub fn edit_config(&self, editor: String, filepath: PathBuf) -> QuartzResult {
        self.edit(&filepath, editor, validator::toml_as::<Config>)?;

        Ok(())
    }
}
