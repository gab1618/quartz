use crate::headers::Headers;

pub struct ResolvedEndpoint {
    pub url: String,
    pub method: String,
    pub headers: Headers,
    pub body: Option<String>,
}

