mod action;
mod cli;
mod editor;

use clap::Parser;
use colored::Colorize;

use crate::cli::{Cli, Cmd};
use quartz_core::{
    QuartzResult,
    ctx::{Ctx, CtxArgs},
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

    let home_dir = std::env::home_dir().unwrap();

    let ctx = Ctx::new(
        std::env::current_dir().unwrap(),
        home_dir,
        CtxArgs {
            from_handle: args.from_handle,
            early_apply_environment: args.apply_environment,
        },
    )?;

    // When true, ensures pagers and/or grep keeps the output colored
    colored::control::set_override(ctx.config.parse().ui.colors());

    action::cmd(ctx, args.command).await?;

    Ok(())
}
