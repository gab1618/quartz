#[derive(Debug, thiserror::Error)]
pub enum CookieError {
    #[error("Could not read cookies: {0}")]
    ReadCookies(#[source] std::io::Error),
    #[error("Could not save cookie: {0}")]
    SaveCookie(#[source] std::io::Error),
    #[error("Cookie has no name")]
    NoNameCookie,
    #[error("Cookie has no value")]
    NoValueCookie,
    #[error("Cookie has no domain")]
    NoDomainCookie,
    #[error("Cookie has invalid format")]
    InvalidFormat,
}
