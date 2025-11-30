use quartz_core::error::QuartzError;

#[derive(Debug, thiserror::Error)]
pub enum QuartzCliError {
    #[error(transparent)]
    QuartzError(#[from] QuartzError),
    #[error("Could not get home dir")]
    GetHomeDir,
    #[error("Could not get current dir: {0}")]
    GetCurrentDir(#[source] std::io::Error),
}

pub type QuartzCliResult<T = ()> = Result<T, QuartzCliError>;
