use crate::{QuartzError, QuartzResult, ctx::Ctx};
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

    pub fn file_path(&self, ctx: &Ctx) -> PathBuf {
        ctx.path().join(Self::STATE_DIR).join(match self {
            Self::Endpoint => "endpoint",
            Self::Env => "env",
            Self::PreviousEndpoint => "previous-endpoint",
        })
    }

    pub fn get(&self, ctx: &Ctx) -> QuartzResult<String> {
        let bytes = std::fs::read(self.file_path(ctx)).map_err(|_| QuartzError::Internal)?;

        Ok(String::from_utf8(bytes).map_err(|_| QuartzError::Internal)?)
    }

    pub fn set(&self, ctx: &Ctx, value: &str) -> QuartzResult {
        let mut file = std::fs::OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(self.file_path(ctx))
            .map_err(|_| QuartzError::Internal)?;

        file.write_all(value.as_bytes())
            .map_err(|_| QuartzError::Internal)
    }
}

impl State {
    pub fn get(&self, ctx: &Ctx, field: StateField) -> QuartzResult<String> {
        let overwrite = match field {
            StateField::Endpoint => self.handle.clone(),
            _ => None,
        };

        if let Some(overwrite) = overwrite {
            return Ok(overwrite);
        }

        field.get(ctx)
    }
}
