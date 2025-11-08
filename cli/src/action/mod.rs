use quartz_core::Quartz;
use quartz_core::QuartzError;

use crate::action::pager::CliPager;
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
pub mod pager;

pub type CliQuartz = Quartz<CliEditor, CliPager>;

pub async fn cmd(ctx: Ctx, command: Cmd) -> QuartzResult {
    let mut quartz = Quartz::from_ctx(ctx, CliEditor::default(), CliPager::default());

    match command {
        Cmd::Init(_) => (), // Init is only run on main, before ctx is resolved

        Cmd::Send(args) => action::send::cmd(quartz, args).await?,
        Cmd::Create(args) => {
            quartz.handle_create(&args.handle, args.patch, args.switch)?;
        }
        Cmd::Use(args) => {
            quartz.handle_switch(args.handle, args.patch, args.empty)?;
        }
        Cmd::Ls(args) => {
            action::ls::cmd(args, quartz)?;
        }
        Cmd::Show { command } => action::show::cmd(&mut quartz.ctx, command)?,
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
        Cmd::Body(args) => action::body::cmd(&mut quartz.ctx, args)?,
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
