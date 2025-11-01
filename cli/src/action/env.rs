use crate::cli::{EnvCmd as Cmd, HeaderCmd};
use quartz_core::{Quartz, QuartzResult};

#[derive(clap::Args, Debug)]
pub struct CreateArgs {
    name: String,
}

#[derive(clap::Args, Debug)]
pub struct CpArgs {
    src: String,
    dest: String,
}

#[derive(clap::Args, Debug)]
pub struct SwitchArgs {
    env: String,
}

#[derive(clap::Args, Debug)]
pub struct RmArgs {
    env: String,
}

pub fn cmd(quartz: Quartz, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Create(args) => quartz.create_env(&args.name)?,
        Cmd::Cp(args) => quartz.cp_env(&args.src, &args.dest)?,
        Cmd::Use(args) => quartz.switch_env(&args.env)?,
        Cmd::Ls => {
            let envs = quartz.get_envs()?;
            for env in envs {
                println!("{env}");
            }
        }
        Cmd::Rm(args) => quartz.remove_env(&args.env)?,
        Cmd::Header { command } => match command {
            HeaderCmd::Set { header } => {
                quartz.header_set(&header)?;
            }
            HeaderCmd::Ls => {
                let curr_env = quartz.current_env();
                println!("{}", curr_env.headers);
            }
            HeaderCmd::Rm { key } => {
                for header in key {
                    quartz.header_rm(&header)?
                }
            }
            HeaderCmd::Get { key } => {
                let header = quartz.header_get(&key)?;
                println!("{header}");
            }
        },
    };

    Ok(())
}
