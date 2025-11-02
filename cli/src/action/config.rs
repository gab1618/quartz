use crate::cli::ConfigCmd as Cmd;
use quartz_core::{Quartz, QuartzResult};

#[derive(clap::Args, Debug)]
pub struct GetArgs {
    key: String,
}

#[derive(clap::Args, Debug)]
pub struct SetArgs {
    key: String,
    value: String,
}

pub fn cmd(mut quartz: Quartz, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Get(args) => {
            let config = quartz.config_get(&args.key)?;
            println!("{config}");
        }
        Cmd::Edit => todo!(),
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
