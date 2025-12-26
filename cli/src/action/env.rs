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
            let env = ctx.quartz.env();
            env.create_env(args.name)?;
        }
        Cmd::Cp(args) => {
            let env = ctx.quartz.env();
            env.cp_env(args.src, args.dest)?;
        }
        Cmd::Use(args) => {
            let env = ctx.quartz.env();
            env.switch_env(args.env)?;
        }
        Cmd::Ls => {
            let env = ctx.quartz.env();
            let envs = env.get_envs()?;
            for env in envs {
                println!("{}", env?);
            }
        }
        Cmd::Rm(args) => {
            let env = ctx.quartz.env();
            env.remove_env(args.env)?;
        },
        Cmd::Header { command } => {
            let env = ctx.quartz.env();
            let mut curr_env = env.env().unwrap();
            match command {
                HeaderCmd::Set { name, value } => {
                    println!("Setting {} to {}", name, value);
                    curr_env.header_set(name, value)?;
                    curr_env.save()?;
                }
                HeaderCmd::Ls => {
                    println!("{}", curr_env.headers);
                }
                HeaderCmd::Rm { key } => {
                    for header in key {
                        curr_env.header_rm(&header)?;
                        curr_env.save()?;
                    }
                }
                HeaderCmd::Get { key } => {
                    let header = curr_env.header_get(&key)?;
                    println!("{header}");
                }
            }
        }
    };

    Ok(())
}
