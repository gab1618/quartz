#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    QuartzError(#[from] quartz_core::error::Error),
    #[error("Could not get home dir")]
    GetHomeDir,
    #[error("Could not get current dir: {0}")]
    GetCurrentDir(#[source] std::io::Error),
    #[error("No handle currently in use")]
    NoHandleInUse,
    #[error("Could not create file to edit: {0}")]
    CreateEditFile(#[source] std::io::Error),
    #[error("Could not read edited file")]
    ReadEditFile(#[source] std::io::Error),
    #[error("Could not copy edit file")]
    CopyEditFile(#[source] std::io::Error),
    #[error("Could not remove edit file")]
    RemoveEditFile(#[source] std::io::Error),
    #[error("Could not parse toml")]
    ParseToml,
}

pub type Result<T = ()> = std::result::Result<T, Error>;
