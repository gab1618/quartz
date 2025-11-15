use crate::cli::SnippetCmd as Cmd;
use quartz_core::{Quartz, QuartzResult, endpoint::EndpointPatch, pairmap::PairMap, snippet};

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

pub fn cmd(quartz: Quartz, mut args: Args) -> QuartzResult {
    let handle = quartz.current_handle().unwrap();
    let mut endpoint = handle.endpoint(&quartz).unwrap();
    let mut env = quartz.current_env()?;

    for var in args.variables {
        env.variables.set(&var)?;
    }

    endpoint.update(&mut args.patch);
    endpoint.apply_env(&env);

    match args.command {
        Cmd::Curl(curl) => curl.print(&mut endpoint)?,
        Cmd::Http => snippet::Http::print(&mut endpoint)?,
    };

    Ok(())
}
