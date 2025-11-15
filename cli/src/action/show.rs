use crate::{action::{self, CliQuartz}, cli::ShowCmd as Cmd};
use quartz_core::{QuartzError, QuartzResult, ctx::Ctx, state::StateField};

pub fn cmd(quartz: CliQuartz, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Query { key } => {
            if let Some(key) = key {
                action::query::get(&quartz.ctx, key);
            } else {
                action::query::print(&quartz.ctx);
            }
        }
        Cmd::Headers { key } => {
            if let Some(key) = key {
                action::header::get(&quartz.ctx, key)?;
            } else {
                action::header::ls(&quartz.ctx)?;
            }
        }
        Cmd::Url => url(&quartz.ctx),
        Cmd::Method => method(&quartz.ctx),
        Cmd::Body => action::body::print(quartz)?,
        Cmd::Handle => handle(&quartz.ctx),
        Cmd::Env => {
            let curr_env = quartz.ctx.require_env();
            println!("{}", curr_env.name);
        }
        Cmd::Cookies(args) => action::cookie::print(&quartz.ctx, args),
        Cmd::Endpoint => endpoint(&quartz.ctx)?,
        Cmd::Snippet(args) => action::snippet::cmd(&quartz.ctx, args)?,
    };

    Ok(())
}

pub fn url(ctx: &Ctx) {
    let (_, endpoint) = ctx.require_endpoint();
    println!("{}", endpoint.url);
}

pub fn method(ctx: &Ctx) {
    let (_, endpoint) = ctx.require_endpoint();
    println!("{}", endpoint.method);
}

pub fn handle(ctx: &Ctx) {
    if let Ok(endpoint) = ctx.state.get(ctx, StateField::Endpoint) {
        println!("{}", endpoint);
    }
}

pub fn endpoint(ctx: &Ctx) -> QuartzResult {
    let (_, endpoint) = ctx.require_endpoint();

    println!("{}", endpoint.to_toml().map_err(|_| QuartzError::Internal)?);
    Ok(())
}
