use crate::{cli::SnippetCmd as Cmd, ctx::Ctx};
use quartz_core::{endpoint::EndpointPatch, error::QuartzResult, pairmap::PairMap, snippet};

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

pub fn cmd(ctx: Ctx, mut args: Args) -> QuartzResult {
    let handle = ctx.quartz.handle().unwrap();
    let mut endpoint = handle.endpoint(&ctx.quartz).unwrap();
    let mut env = ctx.quartz.env()?;

    for var in args.variables {
        env.variables.set(&var)?;
    }

    endpoint.update(&mut args.patch)?;
    endpoint.apply_env(&env);

    match args.command {
        Cmd::Curl(curl) => curl.print(&mut endpoint)?,
        Cmd::Http => snippet::Http::print(&mut endpoint)?,
    };

    Ok(())
}
