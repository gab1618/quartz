use std::{path::PathBuf, str::FromStr as _};

use chrono::Utc;
use hyper::{
    Body, Client, Uri,
    body::{Bytes, HttpBody as _},
    header::{HeaderName, HeaderValue},
};

use crate::{
    Quartz,
    endpoint::{EndpointManager, error::EndpointError},
    env::EnvManager,
    history::{History, error::HistoryError},
};

pub struct Request<'a> {
    path: PathBuf,
    endpoint: EndpointManager<'a>,
    env: EnvManager<'a>,
}

pub const USER_AGENT: &str = concat!("quartz/", env!("CARGO_PKG_VERSION"));

impl<'a> Request<'a> {
    pub fn history(&self) -> History<'_> {
        History::new(&self.path)
    }
    pub async fn send(
        &self,
        no_follow: bool,
        aditional_cookie_jar: Option<PathBuf>,
    ) -> crate::Result<Bytes> {
        let endpoint = &self.endpoint;
        let handle = endpoint.current().ok_or(EndpointError::NoHandleInUse)?;
        let env = &self.env;
        let env = env.current()?;
        let env_value = env.read()?;
        let mut endpoint = handle.endpoint()?;
        let resolved_endpoint = endpoint.as_resolved(&handle, &env_value)?;

        if !endpoint.headers.contains_key("user-agent") {
            endpoint
                .headers
                .insert("user-agent".to_string(), USER_AGENT.to_owned());
        }

        let mut cookie_jar = env.cookie_jar();

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

        let mut entry = crate::history::Entry::builder();
        entry
            .handle(handle.handle())
            .timestamp(Utc::now().timestamp_micros());

        let body = handle.body().clone();

        let mut res: hyper::Response<Body>;

        loop {
            let mut req: hyper::Request<_> = resolved_endpoint
                // TODO: Find a way around this clone
                .clone()
                .try_into()?;
            for (key, val) in env_value.headers.iter() {
                if !endpoint.headers.contains_key(key) {
                    req.headers_mut().insert(
                        HeaderName::from_str(key).map_err(|_| crate::Error::ParseHeader)?,
                        HeaderValue::from_str(val).map_err(|_| crate::Error::ParseHeader)?,
                    );
                }
            }

            if let Some(ref body) = body {
                entry.message_raw(body.to_owned());
            }

            let client = {
                let https = hyper_tls::HttpsConnector::new();
                Client::builder().build(https)
            };

            res = client
                .request(req)
                .await
                .map_err(|_| crate::Error::RequestFailure)?;

            if let Some(cookie_header) = res.headers().get("Set-Cookie") {
                let url = endpoint.full_url()?;

                cookie_jar.set(
                    url.host().unwrap(),
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
                    let url = endpoint.full_url()?;
                    // This is awful
                    endpoint.url = Uri::builder()
                        .authority(url.authority().unwrap().as_str())
                        .scheme(url.scheme().unwrap().as_str())
                        .path_and_query(location)
                        .build()
                        .map_err(|_| crate::Error::ParseLocationHeader)?
                        .to_string();
                } else if Uri::from_str(location).is_ok() {
                    endpoint.url = location.to_string();
                }
            };
        }

        match aditional_cookie_jar {
            Some(path) => cookie_jar.write_at(&path)?,

            None => cookie_jar.write()?,
        };

        let mut bytes = Bytes::new();

        while let Some(chunk) = res.data().await {
            if let Ok(chunk) = chunk {
                bytes = [bytes, chunk].concat().into();
            }
        }

        entry.message_raw(String::from_utf8(bytes.to_vec()).map_err(|_| HistoryError::Serialize)?);

        let h = self.history();
        h.write(entry.build()?)?;

        Ok(bytes)
    }
}

impl<'a> From<&'a Quartz> for Request<'a> {
    fn from(value: &'a Quartz) -> Self {
        Self {
            path: value.path.clone(),
            endpoint: value.endpoint(),
            env: value.env(),
        }
    }
}
