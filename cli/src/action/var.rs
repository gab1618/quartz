use crate::cli::VarCmd as Cmd;
use quartz_core::{ctx::Ctx, editor::Editor, env::Variables, Quartz, QuartzResult};

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

pub fn cmd<E: Editor>(mut quartz: Quartz<E>, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Edit => edit(&mut quartz.ctx)?,
        Cmd::Get(args) => {
            if let Some(v) = quartz.env_var_get(&args.key) {
                print!("{v}");
            }
        }
        Cmd::Set(args) => {
            for variable in args.variables {
                quartz.env_var_set(&variable)?;
            }
        }
        Cmd::Rm(args) => {
            quartz.env_var_rm(args.keys)?;
        }
        Cmd::Ls => {
            let vars = quartz.env_vars_get();
            print!("{vars}");
        }
    };

    Ok(())
}

pub fn edit(ctx: &Ctx) -> QuartzResult {
    let env = ctx.require_env();
    ctx.edit(&env.dir(ctx).join("variables"), |c| {
        Variables::parse(c);
        Ok(())
    })?;

    Ok(())
}
