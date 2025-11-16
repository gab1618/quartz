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
}

pub type QuartzResult<T = ()> = Result<T, QuartzError>;
