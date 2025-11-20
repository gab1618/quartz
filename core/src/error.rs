use crate::endpoint::error::EndpointError;

#[derive(Debug, thiserror::Error)]
pub enum QuartzError {
    #[error(transparent)]
    EndpointError(#[from] EndpointError),
    #[error("Unknown error")]
    Internal,
    #[error("Could not initialize quartz: {0}")]
    Init(#[source] std::io::Error),
    #[error("Quartz already initialized")]
    AlreadyInitialized,
    #[error("Could not setup quartz")]
    Setup,
    #[error("Could not parse header")]
    ParseHeader,
    #[error("Error making the request")]
    RequestFailure,
    #[error("Env already exists")]
    AlreadyExistingEnv,
    #[error("Could not get envs: {0}")]
    GetEnvs(#[source] std::io::Error),
    #[error("Could not parse env filename")]
    ParseEnvName,
    #[error("Could not proceed. Env {0} is in use")]
    EnvInUse(String),
    #[error("Could not delete env: {0}")]
    DeleteEnv(#[source] std::io::Error),
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
    #[error("Could not read state")]
    GetState(#[source] std::io::Error),
    #[error("Could not set state")]
    SetState(#[source] std::io::Error),
    #[error("Could not parse full url")]
    ParseUrl,
    #[error("Could not parse cookie")]
    ParseCookie,
    #[error("Could not parse location header")]
    ParseLocationHeader,
    #[error("Could not save cookie: {0}")]
    SaveCookie(#[source] std::io::Error),
}

pub type QuartzResult<T = ()> = Result<T, QuartzError>;
