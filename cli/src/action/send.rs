use quartz_core::{endpoint::EndpointPatch, Quartz, QuartzResult};
use std::path::PathBuf;
use tokio::io::{AsyncWriteExt, stdout};

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Change a variable when sending the request.
    #[arg(long = "var", short = 'v', value_name = "KEY=VALUE")]
    variables: Vec<String>,

    #[command(flatten)]
    patch: EndpointPatch,

    /// Do not follow redirects
    #[arg(long)]
    no_follow: bool,

    /// Pass cookie data to request header
    #[arg(long = "cookie", short = 'b', value_name = "DATA|FILENAME")]
    cookies: Vec<String>,

    /// Which file to write all cookies after a completed request
    #[arg(long, short = 'c', value_name = "FILE")]
    cookie_jar: Option<PathBuf>,
}

pub async fn cmd(quartz: Quartz, args: Args) -> QuartzResult {
    let bytes = quartz
        .send(
            args.variables,
            args.patch,
            args.no_follow,
            args.cookies,
            args.cookie_jar,
        )
        .await?;

    stdout().write_all(&bytes).await.unwrap();
    Ok(())
}
