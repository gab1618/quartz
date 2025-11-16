#[derive(Debug, thiserror::Error)]
pub enum QuartzError {
    #[error("Unknown error")]
    Internal,
    #[error("Could not initialize quartz: {0}")]
    Init(#[source] std::io::Error),
    #[error("Quartz already initialized")]
    AlreadyInitialized,
    #[error("Could not setup quartz")]
    Setup,
    #[error("Env already exists")]
    AlreadyExistingEnv,
    #[error("Env not found")]
    EnvNotFound,
    #[error("Could not create env dir: {0}")]
    CreateEnvDir(#[source] std::io::Error),
    #[error("Could not update variables file: {0}")]
    UpdateVariablesFile(#[source] std::io::Error),
    #[error("Could not update headers file: {0}")]
    UpdateHeadersFile(#[source] std::io::Error),
    #[error("Header not found")]
    HeaderNotFound,
    #[error("Could not remove header")]
    RemoveHeader,
    #[error("Could not serialize config file: {0}")]
    SerializeConfig(#[source] toml::ser::Error),
    #[error("Invalid config key: {0}")]
    InvalidConfigKey(String),
    #[error("Could not save config: {0}")]
    SaveConfig(#[source] std::io::Error),
    #[error("Could not read cookies: {0}")]
    ReadCookies(#[source] std::io::Error),
    #[error("Could not save handle: {0}")]
    SaveHandle(#[source] std::io::Error),
    #[error("Could not get handle children: {0}")]
    GetHandleChildren(#[source] std::io::Error),
    #[error("Could not parse handle spec")]
    ParseHandleSpec,
    #[error("Could not serialize endpoint: {0}")]
    SerializeEndpoint(#[source] toml::ser::Error),
    #[error("Could not save endpoint: {0}")]
    SaveEndpoint(#[source] std::io::Error),
    #[error("Could not read history entries")]
    ReadHistoryEntries,
    #[error("Could not serialize history")]
    SerializeHistory,
    #[error("Could not save history")]
    SaveHistory,
    #[error("Could not get handle in entrybuilder")]
    GetEntryBuilderHandle,
    #[error("Empty history")]
    EmptyHistory,
    #[error("Could not read history entry")]
    ReadHistoryEntry,
    #[error("Could not parse history entry")]
    ParseHistoryEntry,
    #[error("Could not set value in the keymap")]
    KeymapSet,
}

pub type QuartzResult<T = ()> = Result<T, QuartzError>;
