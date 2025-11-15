use quartz_core::Quartz;
use quartz_core::QuartzError;

use crate::QuartzResult;
use crate::action;
use crate::editor::CliEditor;
use crate::{Ctx, cli::Cmd};

pub mod body;
pub mod config;
pub mod cookie;
pub mod env;
pub mod handle;
pub mod header;
pub mod history;
pub mod init;
pub mod last;
pub mod ls;
pub mod query;
pub mod send;
pub mod show;
pub mod snippet;
pub mod var;

pub type CliQuartz = Quartz<CliEditor>;

pub async fn cmd(ctx: Ctx, command: Cmd) -> QuartzResult {
    let home_dir = std::env::home_dir().ok_or(QuartzError::Internal)?;
    let mut quartz = Quartz::from_ctx(ctx, home_dir, CliEditor::default());

    match command {
        Cmd::Init(_) => (), // Init is only run on main, before ctx is resolved

        Cmd::Send(args) => action::send::cmd(quartz, args).await?,
        Cmd::Create(args) => {
            let endpoint = quartz.handle_create(&args.handle)?;
            if args.switch {
                quartz.handle_switch(args.handle)?;
            }
            quartz.apply_endpoint_patch(endpoint, args.patch)?;
        }
        Cmd::Use(args) => {
            let curr_handle = match args.handle {
                Some(handle) => Some(quartz.handle_switch(handle)?),
                None => quartz.current_handle(),
            };
            let curr_endpoint = curr_handle
                .map(|handle| handle.endpoint(&quartz.ctx))
                .unwrap();
            if let Some(endpoint) = curr_endpoint {
                quartz.apply_endpoint_patch(endpoint, args.patch)?;
            }
            if args.empty {
                quartz.make_handle_empty()?;
            }
        }
        Cmd::Ls(args) => {
            action::ls::cmd(args, quartz)?;
        }
        Cmd::Show { command } => action::show::cmd(quartz, command)?,
        Cmd::Edit => { quartz.handle_edit() }?,
        Cmd::Cp(args) => {
            quartz.handle_cp(args.recursive, args.src, args.dest)?;
        }
        Cmd::Mv(args) => {
            quartz.handle_mv(args.handles)?;
        }
        Cmd::Rm(args) => {
            quartz.handle_rm(args.recursive, args.handles)?;
        }
        Cmd::Query { command } => action::query::cmd(&mut quartz.ctx, command)?,
        Cmd::Header { command } => action::header::cmd(&mut quartz.ctx, command)?,
        Cmd::Body(args) => action::body::cmd(quartz, args)?,
        Cmd::History(args) => action::history::cmd(quartz, args)?,
        Cmd::Last { command } => {
            action::last::cmd(quartz, command).map_err(|_| QuartzError::Internal)?
        }
        Cmd::Var { command } => action::var::cmd(quartz, command)?,
        Cmd::Env { command } => action::env::cmd(quartz, command)?,
        Cmd::Config { command } => action::config::cmd(quartz, command)?,
    };

    Ok(())
}
