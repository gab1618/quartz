use crate::{cli::ConfigCmd as Cmd, ctx::Ctx};
use quartz_core::{config::Config, QuartzResult};

#[derive(clap::Args, Debug)]
pub struct GetArgs {
    key: String,
}

#[derive(clap::Args, Debug)]
pub struct SetArgs {
    key: String,
    value: String,
}

pub fn cmd(ctx: Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Get(args) => {
            let config_manager = ctx.quartz.config();
            let config = config_manager.get(&args.key)?;
            println!("{config}");
        }
        Cmd::Edit => {
            let editor = ctx.quartz.config().parse().preferences.editor();
            let config_file_path = Config::filepath(ctx.quartz.config().path());
            ctx.edit_config(editor, config_file_path)?;
        }
        Cmd::Set(args) => {
            let config_manager = ctx.quartz.config();
            config_manager.set(&args.key, &args.value)?;
        }
        Cmd::Ls => {
            let config_manager = ctx.quartz.config();
            let configs = config_manager.raw_configs()?;
            println!("{configs}");
        }
    };

    Ok(())
}
