use std::str::FromStr as _;

use hyper::{
    Body, Client, Uri,
    body::{Bytes, HttpBody as _},
};

use crate::{
    Quartz,
    cookie::CookieJar,
    endpoint::{error::EndpointError, resolved_endpoint::ResolvedEndpoint},
};

pub struct Request {
    endpoint: ResolvedEndpoint,
    cookie_jar: CookieJar,
}

pub const USER_AGENT: &str = concat!("quartz/", env!("CARGO_PKG_VERSION"));

impl Request {
    pub async fn send(&mut self, no_follow: bool) -> crate::Result<Bytes> {
        let mut res: hyper::Response<Body>;

        loop {
            let req: hyper::Request<_> = self
                .endpoint
                // TODO: Find a way around this clone
                .clone()
                .try_into()?;

            let client = {
                let https = hyper_tls::HttpsConnector::new();
                Client::builder().build(https)
            };

            res = client
                .request(req)
                .await
                .map_err(|_| crate::Error::RequestFailure)?;

            if let Some(cookie_header) = res.headers().get("Set-Cookie") {
                let parsed_uri = Uri::from_str(&self.endpoint.url).unwrap();

                self.cookie_jar.set(
                    parsed_uri.host().unwrap(),
                    cookie_header
                        .to_str()
                        .map_err(|_| crate::Error::ParseCookie)?,
                );
            }

            if no_follow || !res.status().is_redirection() {
                break;
            }

            if let Some(location) = res.headers().get("Location") {
                let location = location
                    .to_str()
                    .map_err(|_| crate::Error::ParseLocationHeader)?;

                if location.starts_with('/') {
                    let parsed_uri = Uri::from_str(&self.endpoint.url).unwrap();
                    // This is awful
                    self.endpoint.url = Uri::builder()
                        .authority(parsed_uri.authority().unwrap().as_str())
                        .scheme(parsed_uri.scheme().unwrap().as_str())
                        .path_and_query(location)
                        .build()
                        .map_err(|_| crate::Error::ParseLocationHeader)?
                        .to_string();
                } else if Uri::from_str(location).is_ok() {
                    self.endpoint.url = location.to_string();
                }
            };
        }

        let mut bytes = Bytes::new();

        while let Some(chunk) = res.data().await {
            if let Ok(chunk) = chunk {
                bytes = [bytes, chunk].concat().into();
            }
        }

        Ok(bytes)
    }
    pub fn cookie_jar(&self) -> &CookieJar {
        &self.cookie_jar
    }
}

impl TryFrom<&Quartz> for Request {
    type Error = crate::Error;

    fn try_from(value: &Quartz) -> Result<Self, Self::Error> {
        let env = value.env();
        let curr_env = env.current()?;
        let env = curr_env.read()?;
        let endpoint = value.endpoint();
        let handle = endpoint.current().ok_or(EndpointError::NoHandleInUse)?;
        let mut endpoint = handle.endpoint()?;
        let resolved = endpoint.as_resolved(&handle, &env)?;

        if !endpoint.headers.contains_key("user-agent") {
            endpoint
                .headers
                .insert("user-agent".to_string(), USER_AGENT.to_owned());
        }

        let cookie_jar = curr_env.cookie_jar();

        let cookie_value = cookie_jar
            .iter()
            .map(|c| format!("{}={}", c.name(), c.value()))
            .collect::<Vec<String>>()
            .join("; ");

        if !cookie_value.is_empty() {
            endpoint
                .headers
                .insert(String::from("Cookie"), cookie_value);
        }

        Ok(Self {
            endpoint: resolved,
            cookie_jar,
        })
    }
}
