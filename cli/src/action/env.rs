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
            env.create(args.name)?;
        }
        Cmd::Cp(args) => {
            let env = ctx.quartz.env();
            env.copy(args.src, args.dest)?;
        }
        Cmd::Use(args) => {
            let env = ctx.quartz.env();
            env.switch(args.env)?;
        }
        Cmd::Ls => {
            let env = ctx.quartz.env();
            let envs = env.envs()?;
            for env in envs {
                println!("{}", env?);
            }
        }
        Cmd::Rm(args) => {
            let env = ctx.quartz.env();
            env.remove(args.env)?;
        }
        Cmd::Header { command } => {
            let env = ctx.quartz.env();
            let curr_env = env.current().unwrap();
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
                    let header = curr_env_value.header_get(&key)?;
                    println!("{header}");
                }
            }
        }
    };

    Ok(())
}
