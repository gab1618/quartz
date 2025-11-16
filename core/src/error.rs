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
    ReadCookies(#[source] std::io::Error)
}

pub type QuartzResult<T = ()> = Result<T, QuartzError>;
