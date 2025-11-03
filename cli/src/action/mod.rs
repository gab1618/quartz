use quartz_core::Quartz;
use quartz_core::QuartzError;

use crate::QuartzResult;
use crate::action;
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

pub async fn cmd(mut ctx: Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Init(_) => (), // Init is only run on main, before ctx is resolved

        Cmd::Send(args) => action::send::cmd(&mut ctx, args).await?,
        Cmd::Create(args) => {
            let quartz = Quartz::from_ctx(ctx);
            quartz.handle_create(&args.handle, args.patch, args.switch)?;
        }
        Cmd::Use(args) => {
            let quartz = Quartz::from_ctx(ctx);
            quartz.handle_switch(args.handle, args.patch, args.empty)?;
        }
        Cmd::Ls(args) => {
            let quartz = Quartz::from_ctx(ctx);
            quartz.handle_ls(args.handle, args.depth);
        },
        Cmd::Show { command } => action::show::cmd(&mut ctx, command)?,
        Cmd::Edit => action::handle::edit(&mut ctx)?,
        Cmd::Cp(args) => {
            let quartz = Quartz::from_ctx(ctx);
            quartz.handle_cp(args.recursive, args.src, args.dest)?;
        }
        Cmd::Mv(args) => {
            let quartz = Quartz::from_ctx(ctx);
            quartz.handle_mv(args.handles)?;
        }
        Cmd::Rm(args) => {
            let quartz = Quartz::from_ctx(ctx);
            quartz.handle_rm(args.recursive, args.handles)?;
        }
        Cmd::Query { command } => action::query::cmd(&mut ctx, command)?,
        Cmd::Header { command } => action::header::cmd(&mut ctx, command)?,
        Cmd::Body(args) => action::body::cmd(&mut ctx, args)?,
        Cmd::History(args) => action::history::cmd(&mut ctx, args)?,
        Cmd::Last { command } => {
            action::last::cmd(&mut ctx, command).map_err(|_| QuartzError::Internal)?
        }
        Cmd::Var { command } => {
            let quartz = Quartz::from_ctx(ctx);
            action::var::cmd(quartz, command)?
        }
        Cmd::Env { command } => {
            let quartz = Quartz::from_ctx(ctx);
            action::env::cmd(quartz, command)?
        }
        Cmd::Config { command } => {
            let quartz = Quartz::from_ctx(ctx);
            action::config::cmd(quartz, command)?
        }
    };

    Ok(())
}
