use crate::{action, cli::ShowCmd as Cmd, ctx::Ctx};
use quartz_core::error::{QuartzError, QuartzResult};

pub fn cmd(ctx: Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Query { key } => {
            if let Some(key) = key {
                action::query::get(ctx, key);
            } else {
                action::query::print(ctx);
            }
        }
        Cmd::Headers { key } => {
            if let Some(key) = key {
                action::header::get(ctx, key)?;
            } else {
                action::header::ls(ctx)?;
            }
        }
        Cmd::Url => url(ctx),
        Cmd::Method => method(ctx),
        Cmd::Body => action::body::print(ctx)?,
        Cmd::Handle => handle(ctx),
        Cmd::Env => {
            let curr_env = ctx.quartz.env().unwrap();
            println!("{}", curr_env.name);
        }
        Cmd::Cookies(args) => action::cookie::print(ctx, args),
        Cmd::Endpoint => endpoint(ctx)?,
        Cmd::Snippet(args) => action::snippet::cmd(ctx, args)?,
    };

    Ok(())
}

pub fn url(ctx: Ctx) {
    let handle = ctx.quartz.handle().unwrap();
    let endpoint = handle.endpoint(&ctx.quartz).unwrap();
    println!("{}", endpoint.url);
}

pub fn method(ctx: Ctx) {
    let handle = ctx.quartz.handle().unwrap();
    let endpoint = handle.endpoint(&ctx.quartz).unwrap();
    println!("{}", endpoint.method);
}

pub fn handle(ctx: Ctx) {
    let handle = ctx.quartz.handle().unwrap();
    println!("{}", handle.head());
}

pub fn endpoint(ctx: Ctx) -> QuartzResult {
    let handle = ctx.quartz.handle().unwrap();
    let endpoint = handle.endpoint(&ctx.quartz).unwrap();

    println!("{}", endpoint.to_toml().map_err(|_| QuartzError::Internal)?);
    Ok(())
}
