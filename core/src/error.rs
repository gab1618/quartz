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
}

pub type QuartzResult<T = ()> = Result<T, QuartzError>;
