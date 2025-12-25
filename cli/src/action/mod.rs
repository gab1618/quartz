use crate::validator;
use quartz_core::endpoint::Endpoint;
use quartz_core::error::Error;

use crate::Result;
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

pub async fn cmd(ctx: Ctx, command: Cmd) -> Result {
    match command {
        Cmd::Init(_) => (), // Init is only run on main, before ctx is resolved

        Cmd::Send(args) => action::send::cmd(ctx, args).await?,
        Cmd::Create(args) => {
            let new_handle = ctx.quartz.new_handle(&args.handle);
            new_handle.write_endpoint(Endpoint::new())?;
            if args.switch {
                ctx.quartz.handle_switch(args.handle)?;
            }

            new_handle.apply_endpoint_patch(args.patch)?;
        }
        Cmd::Use(args) => {
            let curr_handle = match args.handle {
                Some(handle) => Some(ctx.quartz.handle_switch(handle)?),
                None => ctx.quartz.handle(),
            };
            if let Some(handle) = curr_handle {
                handle.apply_endpoint_patch(args.patch)?;

                if args.empty {
                    handle.make_empty();
                }
            }
        }
        Cmd::Ls(args) => {
            action::ls::cmd(ctx, args)?;
        }
        Cmd::Show { command } => action::show::cmd(ctx, command)?,
        Cmd::Edit => {
            if let Some(curr_handle) = ctx.quartz.handle() {
                let endpoint_file_path = curr_handle.endpoint_file_path();
                ctx.edit(&endpoint_file_path, validator::toml_as::<Endpoint>)?;
            }
        }
        Cmd::Cp(args) => {
            ctx.quartz
                .handle_cp(args.recursive, &args.src, &args.dest)?;
        }
        Cmd::Mv(args) => {
            ctx.quartz.handle_mv(&args.src, &args.dest)?;
        }
        Cmd::Rm(args) => {
            for handle in args.handles {
                let handle = ctx.quartz.new_handle(&handle);
                handle.delete(args.recursive)?;
            }
        }
        Cmd::Query { command } => action::query::cmd(ctx, command)?,
        Cmd::Header { command } => action::header::cmd(ctx, command)?,
        Cmd::Body(args) => action::body::cmd(ctx, args)?,
        Cmd::History(args) => action::history::cmd(ctx, args)?,
        Cmd::Last { command } => {
            action::last::cmd(ctx, command).map_err(|_| Error::Internal)?
        }
        Cmd::Var { command } => action::var::cmd(ctx, command)?,
        Cmd::Env { command } => action::env::cmd(ctx, command)?,
        Cmd::Config { command } => action::config::cmd(ctx, command)?,
    };

    Ok(())
}
