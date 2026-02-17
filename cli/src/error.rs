#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Quartz(#[from] quartz_core::error::Error),
    #[error(transparent)]
    Snippet(#[from] quartz_snippet::error::Error),
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
    #[error("There is no history entries")]
    NoHistoryEntry,
    #[error("Could not write to stdin")]
    WriteStdin,
    #[error("Could not spawn pager: {0}")]
    SpawnPager(#[source] std::io::Error),
}

pub type Result<T = ()> = std::result::Result<T, Error>;
