use std::io::stdout;

use crate::{
    cli::SnippetCmd as Cmd,
    ctx::Ctx,
    error::{Error, Result},
};
use quartz_core::pairmap::PairMap;
use quartz_snippet::{Curl, Http};

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Use a new or overwritten variable
    #[arg(long = "var", short = 'v', value_name = "KEY=VALUE")]
    variables: Vec<String>,

    #[command(subcommand)]
    command: crate::cli::SnippetCmd,
}

pub fn cmd(ctx: Ctx, args: Args) -> Result {
    let env = ctx.quartz.env();
    let handle = ctx.quartz.current().ok_or(Error::NoHandleInUse)?;
    let mut endpoint = handle.endpoint()?;
    let curr_env = env.current()?;
    let mut curr_env_value = curr_env.read()?;

    for var in args.variables {
        curr_env_value.variables.set(&var)?;
    }

    let resolved = endpoint.as_resolved(&handle, &curr_env_value)?;

    let mut stdout = stdout();
    match args.command {
        Cmd::Curl { long, multiline } => Curl::write(&mut stdout, resolved, long, multiline)?,
        Cmd::Http => Http::write(&mut stdout, resolved)?,
    };

    Ok(())
}
