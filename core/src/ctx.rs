use std::ffi::OsString;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use colored::Colorize;

use crate::config::ConfigManager;
use crate::endpoint::{Endpoint, EndpointHandle};
use crate::env::Env;
use crate::state::{State, StateField};
use crate::{QuartzError, QuartzResult};

pub struct CtxArgs {
    pub from_handle: Option<String>,
    pub early_apply_environment: bool,
}

pub struct Ctx {
    pub config: ConfigManager,
    pub state: State,
    path: PathBuf,
    code: ExitCode,
}

impl Ctx {
    pub fn new(mut dir: PathBuf, config_path: PathBuf) -> QuartzResult<Self> {
        let config = ConfigManager::new(config_path);
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
            config,
            state,
            path: dir.join(".quartz"),
            code: ExitCode::default(),
        })
    }

    pub fn require_input_handle(&self, handle: &str) -> EndpointHandle {
        let result = EndpointHandle::from(handle);

        if !result.exists(self) {
            panic!("could not find {} handle", handle.red());
        }

        result
    }

    pub fn require_handle(&self) -> EndpointHandle {
        let mut result = None;
        if let Ok(handle) = self.state.get(self, StateField::Endpoint) {
            if !handle.is_empty() {
                result = Some(EndpointHandle::from(handle));
            }
        }

        match result {
            Some(handle) => handle,
            None => panic!("no handle in use. Try {}", "quartz use <HANDLE>".green()),
        }
    }

    pub fn require_endpoint(&self) -> (EndpointHandle, Endpoint) {
        let handle = self.require_handle();
        let endpoint = self.require_endpoint_from_handle(&handle);

        (handle, endpoint)
    }

    pub fn require_endpoint_from_handle(&self, handle: &EndpointHandle) -> Endpoint {
        let endpoint = handle.endpoint(self).unwrap_or_else(|| {
            panic!("no endpoint at {}", handle.handle().red());
        });

        endpoint
    }

    /// Returns current env.
    ///
    /// # Panics
    ///
    /// Program is terminated if it is unable to require it.
    pub fn require_env(&self) -> Env {
        let state = self
            .state
            .get(self, StateField::Env)
            .unwrap_or("default".into());

        Env::parse(self.path().to_path_buf(), &state)
            .unwrap_or_else(|_| panic!("could not resolve {} environment", state.red()))
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
        self.edit_with_extension::<F>(path, None, validate)
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

        let editor = self.config.parse().preferences.editor();
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
}
