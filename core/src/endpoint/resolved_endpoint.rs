use hyper::{Body, Request};

use crate::{endpoint::error::EndpointError, error::Error, headers::Headers};

#[derive(Clone)]
pub struct ResolvedEndpoint {
    pub url: String,
    pub method: String,
    pub headers: Headers,
    pub body: Option<String>,
}

impl TryInto<Request<Body>> for ResolvedEndpoint {
    type Error = Error;

    fn try_into(self) -> Result<Request<Body>, Self::Error> {
        let mut builder = hyper::Request::builder().uri(&self.url);

        if let Ok(method) = hyper::Method::from_bytes(self.method.as_bytes()) {
            builder = builder.method(method);
        }

        for (key, value) in self.headers.iter() {
            builder = builder.header(key, value);
        }

        if let Some(body) = self.body {
            builder
                .body(body.to_owned().into())
                .map_err(|_| EndpointError::SetRequestBody.into())
        } else {
            builder
                .body(Body::empty())
                .map_err(|_| EndpointError::SetRequestBody.into())
        }
    }
}
