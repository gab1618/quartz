use crate::{
    config::error::ConfigError, cookie::error::CookieError, endpoint::error::EndpointError,
    env::error::EnvError, history::error::HistoryError, request::error::RequestError,
    state::error::StateError,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not initialize quartz: {0}")]
    Init(#[source] std::io::Error),
    #[error("Quartz already initialized")]
    AlreadyInitialized,
    #[error("Could not setup quartz")]
    Setup,
    #[error("Could not set value in the keymap")]
    KeymapSet,
    #[error(transparent)]
    EndpointError(#[from] EndpointError),
    #[error(transparent)]
    HistoryError(#[from] HistoryError),
    #[error(transparent)]
    EnvError(#[from] EnvError),
    #[error(transparent)]
    RequestError(#[from] RequestError),
    #[error(transparent)]
    StateError(#[from] StateError),
    #[error(transparent)]
    ConfigErr(#[from] ConfigError),
    #[error(transparent)]
    CookieErr(#[from] CookieError),
}

pub type Result<T = ()> = std::result::Result<T, Error>;
