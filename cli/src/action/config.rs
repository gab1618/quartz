use crate::cli::ConfigCmd as Cmd;
use quartz_core::{config::Config, Quartz, QuartzResult};

#[derive(clap::Args, Debug)]
pub struct GetArgs {
    key: String,
}

#[derive(clap::Args, Debug)]
pub struct SetArgs {
    key: String,
    value: String,
}

pub fn cmd(quartz: Quartz, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Get(args) => {
            let config_manager = quartz.config();
            let config = config_manager.get(&args.key)?;
            println!("{config}");
        }
        Cmd::Edit => {
            let editor = quartz.config().parse().preferences.editor();
            let config_file_path = Config::filepath(quartz.config().path());
            quartz.ctx.edit_config(editor, config_file_path)?;
        }
        Cmd::Set(args) => {
            let config_manager = quartz.config();
            config_manager.set(&args.key, &args.value)?;
        }
        Cmd::Ls => {
            let config_manager = quartz.config();
            let configs = config_manager.raw_configs()?;
            println!("{configs}");
        }
    };

    Ok(())
}
