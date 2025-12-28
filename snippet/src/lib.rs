pub mod error;

pub use crate::error::{Error, Result};
use std::{io::Write, ops::Deref};

use quartz_core::endpoint::resolved_endpoint::ResolvedEndpoint;

use hyper::{Body, Request, Response, Uri};

enum CurlOption {
    Location,
    Request,
    Header,
    Data,
}

pub struct Curl;

impl Curl {
    pub fn write<W: Write>(
        w: &mut W,
        endpoint: ResolvedEndpoint,
        long: bool,
        multiline: bool,
    ) -> Result {
        let separator = if multiline { " \\\n\t" } else { " " };

        write!(
            w,
            "curl {} '{}'",
            Self::option_string(CurlOption::Location, long),
            endpoint.url
        )
        .map_err(Error::WriteSnippet)?;
        write!(
            w,
            " {} {}",
            Self::option_string(CurlOption::Request, long),
            endpoint.method
        )
        .map_err(Error::WriteSnippet)?;

        for (key, value) in endpoint.headers.iter() {
            write!(
                w,
                "{}{} '{}: {}'",
                separator,
                Self::option_string(CurlOption::Header, long),
                key,
                value
            )
            .map_err(Error::WriteSnippet)?;
        }

        if let Some(body) = endpoint.body {
            let mut body = body.to_owned();
            write!(w, "{}{} '", separator, Self::option_string(CurlOption::Data, long))
                .map_err(Error::WriteSnippet)?;

            if body.ends_with('\n') {
                body.truncate(body.len() - 1);
            }

            write!(w, "{body}").map_err(Error::WriteSnippet)?;
            writeln!(w, "'").map_err(Error::WriteSnippet)?;
        } else {
            writeln!(w, "").map_err(Error::WriteSnippet)?;
        }

        Ok(())
    }

    fn option_string(option: CurlOption, long: bool) -> String {
        let result = match option {
            CurlOption::Location => {
                if long {
                    "--location"
                } else {
                    "-L"
                }
            }
            CurlOption::Request => {
                if long {
                    "--request"
                } else {
                    "-X"
                }
            }
            CurlOption::Header => {
                if long {
                    "--header"
                } else {
                    "-H"
                }
            }
            CurlOption::Data => {
                if long {
                    "--data"
                } else {
                    "-d"
                }
            }
        };

        result.to_string()
    }
}

pub struct Http(String);

impl Deref for Http {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&Response<Body>> for Http {
    fn from(value: &Response<Body>) -> Self {
        let mut output = String::new();

        output.push_str(&format!("< {:?}", value.version()));
        output.push_str(&format!(" {:?}", value.status()));
        output.push('\n');

        for (k, v) in value.headers().iter() {
            output.push_str(&format!(
                "< {}: {}\n",
                k.as_str(),
                v.to_str().unwrap_or_default()
            ))
        }

        output.push('<');

        Self(output)
    }
}

impl From<&Request<Body>> for Http {
    fn from(value: &Request<Body>) -> Self {
        let mut output = String::new();

        output.push_str(&format!(
            "> {} {} {:?}\n",
            value.method(),
            value.uri().path_and_query().unwrap().as_str(),
            value.version()
        ));
        output.push_str(&format!("> Host: {}\n", value.uri().host().unwrap()));

        for (k, v) in value.headers().iter() {
            output.push_str(&format!(
                "> {}: {}\n",
                k.as_str(),
                v.to_str().unwrap_or_default()
            ))
        }

        output.push('>');

        Self(output)
    }
}

impl Http {
    pub fn write<W: Write>(w: &mut W, endpoint: ResolvedEndpoint) -> Result {
        let url: Uri = endpoint.url.try_into().map_err(|_| Error::SerializeUri)?;
        let path = url.path_and_query().ok_or(Error::GetUriPath)?;

        writeln!(w, "{} {} HTTP/1.1", endpoint.method, path.as_str())
            .map_err(Error::WriteSnippet)?;
        writeln!(w, "Host: {}", url.host().ok_or(Error::NoHostFound)?)
            .map_err(Error::WriteSnippet)?;
        write!(w, "{}", endpoint.headers).map_err(Error::WriteSnippet)?;

        if let Some(body) = endpoint.body {
            writeln!(w, "").map_err(Error::WriteSnippet)?;
            write!(w, "{body}").map_err(Error::WriteSnippet)?;
        }

        Ok(())
    }
}
