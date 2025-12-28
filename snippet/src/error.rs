#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not write snippet: {0}")]
    WriteSnippet(#[source] std::io::Error),
    #[error("Could not serialize uri")]
    SerializeUri,
    #[error("Could not get uri path")]
    GetUriPath,
    #[error("Could not get host from resolved endpoint")]
    NoHostFound,
}

pub type Result<T = ()> = std::result::Result<T, Error>;
