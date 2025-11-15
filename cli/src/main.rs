mod action;
mod cli;

use std::env::current_dir;

use clap::Parser;
use colored::Colorize;

use crate::cli::{Cli, Cmd};
use quartz_core::{Quartz, QuartzError, QuartzResult};

#[tokio::main]
async fn main() -> QuartzResult {
    std::panic::set_hook(Box::new(|info| {
        let payload = if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else {
            info.to_string()
        };

        eprintln!("{}: {payload}", "error".red().bold());
    }));

    let args = Cli::parse();

    // Has to run outside action flow because it cannot resolve `ctx`.
    if let Cmd::Init(args) = args.command {
        action::init::cmd(args)?;
        return Ok(());
    }

    let home_dir = std::env::home_dir().unwrap();
    let curr_dir = current_dir().map_err(|_| QuartzError::Internal)?;

    let quartz = Quartz::new(curr_dir, home_dir)?;

    // When true, ensures pagers and/or grep keeps the output colored
    colored::control::set_override(quartz.config().parse().ui.colors());

    action::cmd(quartz, args.command).await?;

    Ok(())
}
