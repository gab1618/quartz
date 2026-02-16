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
    #[error("Could not build URI")]
    BuildURI,
    #[error("Could not parse location header")]
    ParseLocationHeader,
    #[error("Could not parse cookie")]
    ParseCookie,
    #[error("Error making the request")]
    RequestFailure,
}
