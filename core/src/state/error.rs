#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error("Could not read state")]
    GetState(#[source] std::io::Error),
    #[error("Could not set state")]
    SetState(#[source] std::io::Error),
}
