use crate::cli::VarCmd as Cmd;
use quartz_core::{Quartz, QuartzResult, env::Variables};

#[derive(clap::Args, Debug)]
pub struct GetArgs {
    key: String,
}

#[derive(clap::Args, Debug)]
pub struct SetArgs {
    #[arg(name = "VARIABLE", required = true)]
    variables: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct RmArgs {
    #[arg(name = "KEY", required = true)]
    keys: Vec<String>,
}

pub fn cmd(quartz: Quartz, command: Cmd) -> QuartzResult {
    let mut curr_env = quartz.current_env()?;
    match command {
        Cmd::Edit => edit(quartz)?,
        Cmd::Get(args) => {
            if let Some(v) = curr_env.var_get(&args.key) {
                print!("{v}");
            }
        }
        Cmd::Set(args) => {
            for variable in args.variables {
                curr_env.var_set(&variable)?;
            }
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

pub fn edit(quartz: Quartz) -> QuartzResult {
    let env = quartz.current_env()?;
    quartz.ctx.edit(&env.dir().join("variables"), |c| {
        Variables::parse(c);
        Ok(())
    })?;

    Ok(())
}
