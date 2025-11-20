mod action;
mod cli;
mod ctx;

use std::env::current_dir;

use clap::Parser;
use colored::Colorize;

use crate::{
    cli::{Cli, Cmd},
    ctx::Ctx,
};
use quartz_core::{
    Quartz,
    error::{QuartzError, QuartzResult},
};

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

    let home_dir = std::env::home_dir().expect("Could not get home dir");
    let curr_dir = current_dir().expect("Could not get current");

    let quartz = Quartz::new(curr_dir, home_dir)?;
    if let Some(handle) = args.from_handle {
        quartz.handle_switch(handle)?;
    }
    colored::control::set_override(quartz.config().parse().ui.colors());

    let ctx = Ctx::new(quartz);

    // When true, ensures pagers and/or grep keeps the output colored

    action::cmd(ctx, args.command).await?;

    Ok(())
}
