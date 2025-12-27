use std::io::stdout;

use crate::{cli::SnippetCmd as Cmd, ctx::Ctx};
use quartz_core::{endpoint::endpoint::EndpointPatch, error::Result, pairmap::PairMap, snippet};

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Use a new or overwritten variable
    #[arg(long = "var", short = 'v', value_name = "KEY=VALUE")]
    variables: Vec<String>,

    #[command(flatten)]
    patch: EndpointPatch,

    #[command(subcommand)]
    command: crate::cli::SnippetCmd,
}

pub fn cmd(ctx: Ctx, mut args: Args) -> Result {
    let endpoint = ctx.quartz.endpoint();
    let env = ctx.quartz.env();
    let handle = endpoint.current().unwrap();
    let mut endpoint = handle.endpoint().unwrap();
    let curr_env = env.current()?;
    let mut curr_env_value = curr_env.read()?;

    for var in args.variables {
        curr_env_value.variables.set(&var)?;
    }

    endpoint.update(&mut args.patch)?;
    let resolved = endpoint.as_resolved(&handle, &curr_env_value)?;

    let mut stdout = stdout();
    match args.command {
        Cmd::Curl(curl) => curl.write(&mut stdout, resolved)?,
        Cmd::Http => snippet::Http::write(&mut stdout, resolved)?,
    };

    Ok(())
}
