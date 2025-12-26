use quartz_core::error::Result;
use std::path::PathBuf;
use tokio::io::{AsyncWriteExt, stdout};

use crate::ctx::Ctx;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Do not follow redirects
    #[arg(long)]
    no_follow: bool,

    /// Which file to write all cookies after a completed request
    #[arg(long, short = 'c', value_name = "FILE")]
    cookie_jar: Option<PathBuf>,
}

pub async fn cmd(ctx: Ctx, args: Args) -> Result {
    let bytes = ctx.quartz.send(args.no_follow, args.cookie_jar).await?;

    stdout().write_all(&bytes).await.unwrap();
    Ok(())
}
