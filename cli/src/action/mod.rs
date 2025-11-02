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
        Cmd::Create(args) => action::handle::create(&mut ctx, args),
        Cmd::Use(args) => action::handle::switch(&mut ctx, args),
        Cmd::Ls(args) => action::ls::cmd(&mut ctx, args),
        Cmd::Show { command } => action::show::cmd(&mut ctx, command)?,
        Cmd::Edit => action::handle::edit(&mut ctx)?,
        Cmd::Cp(args) => action::handle::cp(&mut ctx, args)?,
        Cmd::Mv(args) => action::handle::mv(&mut ctx, args)?,
        Cmd::Rm(args) => action::handle::rm(&mut ctx, args)?,
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
        },
        Cmd::Env { command } => {
            let quartz = Quartz::from_ctx(ctx);
            action::env::cmd(quartz, command)?
        },
        Cmd::Config { command } => action::config::cmd(&mut ctx, command)?,
    };

    Ok(())
}
