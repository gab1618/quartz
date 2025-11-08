use crate::{action::CliQuartz, cli::ConfigCmd as Cmd};
use quartz_core::QuartzResult;

#[derive(clap::Args, Debug)]
pub struct GetArgs {
    key: String,
}

#[derive(clap::Args, Debug)]
pub struct SetArgs {
    key: String,
    value: String,
}

pub fn cmd(quartz: CliQuartz, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Get(args) => {
            let config = quartz.config_get(&args.key)?;
            println!("{config}");
        }
        Cmd::Edit => {
            quartz.edit_config()?;
        }
        Cmd::Set(args) => {
            quartz.config_set(&args.key, &args.value)?;
        }
        Cmd::Ls => {
            let configs = quartz.config_ls()?;
            println!("{configs}");
        }
    };

    Ok(())
}
