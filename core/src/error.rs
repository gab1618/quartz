use crate::{config::error::ConfigError, endpoint::error::EndpointError, env::error::EnvError, history::error::HistoryError};

#[derive(Debug, thiserror::Error)]
pub enum QuartzError {
    #[error(transparent)]
    EndpointError(#[from] EndpointError),
    #[error(transparent)]
    HistoryError(#[from] HistoryError),
    #[error(transparent)]
    EnvError(#[from] EnvError),
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
    #[error("Header not found")]
    HeaderNotFound,
    #[error("Could not remove header")]
    RemoveHeader,
    #[error(transparent)]
    ConfigErr(#[from] ConfigError),
    #[error("Could not read cookies: {0}")]
    ReadCookies(#[source] std::io::Error),
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
    #[error("Could write snippet: {0}")]
    WriteSnippet(#[source] std::io::Error),
}

pub type QuartzResult<T = ()> = Result<T, QuartzError>;
