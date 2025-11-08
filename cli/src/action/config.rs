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
            let config_manager = quartz.config();
            let config = config_manager .get(&args.key)?;
            println!("{config}");
        }
        Cmd::Edit => {
            quartz.edit_config()?;
        }
        Cmd::Set(args) => {
            let config_manager = quartz.config();
            config_manager.set(&args.key, &args.value)?;
        }
        Cmd::Ls => {
            let config_manager = quartz.config();
            let configs = config_manager.raw_configs()?;
            println!("{configs}");
        }
    };

    Ok(())
}
