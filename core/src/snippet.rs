use std::{io::Write, ops::Deref};

use crate::{QuartzError, QuartzResult, endpoint::resolved_endpoint::ResolvedEndpoint};
use hyper::{Body, Request, Response, Uri};

enum CurlOption {
    Location,
    Request,
    Header,
    Data,
}

#[derive(clap::Args, Debug)]
pub struct Curl {
    /// Use long form cURL options (--header instead of -H)
    #[arg(long)]
    long: bool,

    /// Split output across multiple lines
    #[arg(long)]
    multiline: bool,
}

impl Curl {
    pub fn write<W: Write>(&self, w: &mut W, endpoint: ResolvedEndpoint) -> QuartzResult {
        let separator = if self.multiline { " \\\n\t" } else { " " };

        write!(
            w,
            "curl {} '{}'",
            self.option_string(CurlOption::Location),
            endpoint.url
        )
        .map_err(QuartzError::WriteSnippet)?;
        write!(
            w,
            " {} {}",
            self.option_string(CurlOption::Request),
            endpoint.method
        )
        .map_err(QuartzError::WriteSnippet)?;

        for (key, value) in endpoint.headers.iter() {
            write!(
                w,
                "{}{} '{}: {}'",
                separator,
                self.option_string(CurlOption::Header),
                key,
                value
            )
            .map_err(QuartzError::WriteSnippet)?;
        }

        if let Some(body) = endpoint.body {
            let mut body = body.to_owned();
            write!(w, "{}{} '", separator, self.option_string(CurlOption::Data))
                .map_err(QuartzError::WriteSnippet)?;

            if body.ends_with('\n') {
                body.truncate(body.len() - 1);
            }

            write!(w, "{body}").map_err(QuartzError::WriteSnippet)?;
            writeln!(w, "'").map_err(QuartzError::WriteSnippet)?;
        } else {
            writeln!(w, "").map_err(QuartzError::WriteSnippet)?;
        }

        Ok(())
    }

    fn option_string(&self, option: CurlOption) -> String {
        let result = match option {
            CurlOption::Location => {
                if self.long {
                    "--location"
                } else {
                    "-L"
                }
            }
            CurlOption::Request => {
                if self.long {
                    "--request"
                } else {
                    "-X"
                }
            }
            CurlOption::Header => {
                if self.long {
                    "--header"
                } else {
                    "-H"
                }
            }
            CurlOption::Data => {
                if self.long {
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
    pub fn write<W: Write>(w: &mut W, endpoint: ResolvedEndpoint) -> QuartzResult {
        let url: Uri = endpoint.url.try_into().map_err(|_| QuartzError::Internal)?;
        let path = url.path_and_query().unwrap();

        writeln!(w, "{} {} HTTP/1.1", endpoint.method, path.as_str())
            .map_err(QuartzError::WriteSnippet)?;
        writeln!(w, "Host: {}", url.host().unwrap()).map_err(QuartzError::WriteSnippet)?;
        write!(w, "{}", endpoint.headers).map_err(QuartzError::WriteSnippet)?;

        if let Some(body) = endpoint.body {
            writeln!(w, "").map_err(QuartzError::WriteSnippet)?;
            write!(w, "{body}").map_err(QuartzError::WriteSnippet)?;
        }

        Ok(())
    }
}
