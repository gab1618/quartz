use quartz_core::endpoint::Endpoint;
use quartz_core::error::QuartzError;
use quartz_core::validator;

use crate::QuartzResult;
use crate::action;
use crate::cli::Cmd;
use crate::ctx::Ctx;

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

pub async fn cmd(ctx: Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Init(_) => (), // Init is only run on main, before ctx is resolved

        Cmd::Send(args) => action::send::cmd(ctx, args).await?,
        Cmd::Create(args) => {
            let endpoint = ctx.quartz.handle_create(&args.handle)?;
            if args.switch {
                ctx.quartz.handle_switch(args.handle)?;
            }
            ctx.quartz.apply_endpoint_patch(endpoint, args.patch)?;
        }
        Cmd::Use(args) => {
            let curr_handle = match args.handle {
                Some(handle) => Some(ctx.quartz.handle_switch(handle)?),
                None => ctx.quartz.handle(),
            };
            let curr_endpoint = curr_handle
                .map(|handle| handle.endpoint(&ctx.quartz))
                .unwrap();
            if let Some(endpoint) = curr_endpoint {
                ctx.quartz.apply_endpoint_patch(endpoint, args.patch)?;
            }
            if args.empty {
                ctx.quartz.make_handle_empty()?;
            }
        }
        Cmd::Ls(args) => {
            action::ls::cmd(ctx, args)?;
        }
        Cmd::Show { command } => action::show::cmd(ctx, command)?,
        Cmd::Edit => {
            let endpoint_path = ctx
                .quartz
                .handle_endpoint_file_path()
                .ok_or(QuartzError::Internal)?;

            ctx.edit(&endpoint_path, validator::toml_as::<Endpoint>)?;

            Ok(())
        }?,
        Cmd::Cp(args) => {
            ctx.quartz.handle_cp(args.recursive, args.src, args.dest)?;
        }
        Cmd::Mv(args) => {
            ctx.quartz.handle_mv(args.handles)?;
        }
        Cmd::Rm(args) => {
            ctx.quartz.handle_rm(args.recursive, args.handles)?;
        }
        Cmd::Query { command } => action::query::cmd(ctx, command)?,
        Cmd::Header { command } => action::header::cmd(ctx, command)?,
        Cmd::Body(args) => action::body::cmd(ctx, args)?,
        Cmd::History(args) => action::history::cmd(ctx, args)?,
        Cmd::Last { command } => {
            action::last::cmd(ctx, command).map_err(|_| QuartzError::Internal)?
        }
        Cmd::Var { command } => action::var::cmd(ctx, command)?,
        Cmd::Env { command } => action::env::cmd(ctx, command)?,
        Cmd::Config { command } => action::config::cmd(ctx, command)?,
    };

    Ok(())
}
