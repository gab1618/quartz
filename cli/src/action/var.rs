use crate::error::Result;
use crate::{cli::VarCmd as Cmd, ctx::Ctx};
use quartz_core::env::value::Variables;

#[derive(clap::Args, Debug)]
pub struct GetArgs {
    key: String,
}

#[derive(clap::Args, Debug)]
pub struct SetArgs {
    #[arg(name = "key", required = true)]
    key: String,
    #[arg(name = "value", required = true)]
    value: String,
}

#[derive(clap::Args, Debug)]
pub struct RmArgs {
    #[arg(name = "KEY", required = true)]
    keys: Vec<String>,
}

pub fn cmd(ctx: Ctx, command: Cmd) -> Result {
    let env = ctx.quartz.env();
    let curr_env = env.current()?;
    let mut curr_env_value = curr_env.read()?;
    match command {
        Cmd::Edit => edit(ctx)?,
        Cmd::Get(args) => {
            if let Some(v) = curr_env_value.var_get(&args.key) {
                print!("{v}");
            }
        }
        Cmd::Set(args) => {
            curr_env_value.var_set(args.key, args.value)?;
            curr_env.save(&curr_env_value)?;
        }
        Cmd::Rm(args) => {
            curr_env_value.var_rm(args.keys)?;
            curr_env.save(&curr_env_value)?;
        }
        Cmd::Ls => {
            let vars = curr_env_value.vars();
            print!("{vars}");
        }
    };

    Ok(())
}

pub fn edit(ctx: Ctx) -> Result {
    let env = ctx.quartz.env();
    let curr_env = env.current()?;
    ctx.edit(&curr_env.dir().join("variables"), |c| {
        Variables::parse(c)?;
        Ok(())
    })?;

    Ok(())
}
