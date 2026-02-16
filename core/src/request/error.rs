#[derive(Debug, thiserror::Error)]
pub enum RequestError {
    #[error("Could not parse endpoint URI")]
    ParseEndpointURI,
    #[error("URI has no host")]
    NoHostInURI,
    #[error("URI has no authority")]
    NoAuthorityInURI,
    #[error("URI has no scheme")]
    NoSchemeInURI,
}
