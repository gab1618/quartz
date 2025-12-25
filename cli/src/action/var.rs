use crate::error::Result;
use crate::{cli::VarCmd as Cmd, ctx::Ctx};
use quartz_core::env::env::Variables;

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
    let mut curr_env = ctx.quartz.env()?;
    match command {
        Cmd::Edit => edit(ctx)?,
        Cmd::Get(args) => {
            if let Some(v) = curr_env.var_get(&args.key) {
                print!("{v}");
            }
        }
        Cmd::Set(args) => {
            curr_env.var_set(args.key, args.value)?;
        }
        Cmd::Rm(args) => {
            curr_env.var_rm(args.keys)?;
        }
        Cmd::Ls => {
            let vars = curr_env.vars();
            print!("{vars}");
        }
    };

    Ok(())
}

pub fn edit(ctx: Ctx) -> Result {
    let env = ctx.quartz.env()?;
    ctx.edit(&env.dir().join("variables"), |c| {
        Variables::parse(c)?;
        Ok(())
    })?;

    Ok(())
}
