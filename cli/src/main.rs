mod action;
mod cli;
mod ctx;
mod error;
mod validator;

use std::env::current_dir;

use clap::Parser;

use crate::{
    cli::{Cli, Cmd},
    ctx::Ctx,
    error::{QuartzCliError, QuartzCliResult},
};
use quartz_core::{
    Quartz,
    error::{Error, Result},
};

async fn entrypoint() -> QuartzCliResult {
    let args = Cli::parse();

    // Has to run outside action flow because it cannot resolve `ctx`.
    if let Cmd::Init(args) = args.command {
        action::init::cmd(args)?;
        return Ok(());
    }

    let home_dir = std::env::home_dir().ok_or(QuartzCliError::GetHomeDir)?;
    let curr_dir = current_dir().map_err(QuartzCliError::GetCurrentDir)?;

    let quartz = Quartz::new(curr_dir, home_dir)?;

    // When true, ensures pagers and/or grep keeps the output colored
    colored::control::set_override(quartz.config().parse().ui.colors());

    let ctx = Ctx::new(quartz);

    action::cmd(ctx, args.command).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result {
    if let Err(error) = entrypoint().await {
        eprintln!("{}", error);
        std::process::exit(1);
    }
    Ok(())
}
