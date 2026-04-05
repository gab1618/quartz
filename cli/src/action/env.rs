use crate::{
    cli::{EnvCmd as Cmd, HeaderCmd},
    ctx::Ctx,
};
use quartz_core::error::Result;

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

pub fn cmd(ctx: Ctx, command: Cmd) -> Result {
    match command {
        Cmd::Create(args) => {
            ctx.quartz.create_env(args.name)?;
        }
        Cmd::Cp(args) => {
            ctx.quartz.copy_env(args.src, args.dest)?;
        }
        Cmd::Use(args) => {
            ctx.quartz.switch_env(args.env)?;
        }
        Cmd::Ls => {
            let envs = ctx.quartz.envs()?;
            for env in envs {
                println!("{}", env?);
            }
        }
        Cmd::Rm(args) => {
            ctx.quartz.remove_env(args.env)?;
        }
        Cmd::Header { command } => {
            let curr_env = ctx.quartz.current_env()?;
            let mut curr_env_value = curr_env.read()?;
            match command {
                HeaderCmd::Set { name, value } => {
                    println!("Setting {} to {}", name, value);
                    curr_env_value.header_set(name, value)?;
                    curr_env.save(&curr_env_value)?;
                }
                HeaderCmd::Ls => {
                    println!("{}", curr_env_value.headers);
                }
                HeaderCmd::Rm { key } => {
                    for header in key {
                        curr_env_value.header_rm(&header)?;
                        curr_env.save(&curr_env_value)?;
                    }
                }
                HeaderCmd::Get { key } => {
                    if let Some(header) = curr_env_value.header_get(&key) {
                        println!("{header}");
                    }
                }
            }
        }
    };

    Ok(())
}
