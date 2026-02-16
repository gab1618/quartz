use std::{path::PathBuf, str::FromStr as _};

use chrono::Utc;
use hyper::{
    Body, Client, Uri,
    body::{Bytes, HttpBody as _},
    header::{HeaderName, HeaderValue},
};

use crate::{
    Quartz,
    cookie::CookieJar,
    endpoint::{error::EndpointError, resolved_endpoint::ResolvedEndpoint},
    history::{History, error::HistoryError},
};

pub struct Request {
    path: PathBuf,
    endpoint: ResolvedEndpoint,
    body: Option<String>,
    handle_name: String,
    cookie_jar: CookieJar,
}

pub const USER_AGENT: &str = concat!("quartz/", env!("CARGO_PKG_VERSION"));

impl Request {
    pub fn history(&self) -> History<'_> {
        History::new(&self.path)
    }
    pub async fn send(
        &mut self,
        no_follow: bool,
        aditional_cookie_jar: Option<PathBuf>,
    ) -> crate::Result<Bytes> {
        let mut entry = crate::history::Entry::builder();
        entry
            .handle(self.handle_name.clone())
            .timestamp(Utc::now().timestamp_micros());

        let mut res: hyper::Response<Body>;

        loop {
            let mut req: hyper::Request<_> = self
                .endpoint
                // TODO: Find a way around this clone
                .clone()
                .try_into()?;
            for (key, val) in self.endpoint.headers.iter() {
                if !self.endpoint.headers.contains_key(key) {
                    req.headers_mut().insert(
                        HeaderName::from_str(key).map_err(|_| crate::Error::ParseHeader)?,
                        HeaderValue::from_str(val).map_err(|_| crate::Error::ParseHeader)?,
                    );
                }
            }

            if let Some(ref body) = self.body {
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

        match aditional_cookie_jar {
            Some(path) => self.cookie_jar.write_at(&path)?,

            None => self.cookie_jar.write()?,
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
            path: value.path.clone(),
            endpoint: resolved,
            body: handle.body(),
            handle_name: handle.handle(),
            cookie_jar,
        })
    }
}
