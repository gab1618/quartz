use crate::{
    action,
    cli::ShowCmd as Cmd,
    ctx::Ctx,
    error::{Error, Result},
};

pub fn cmd(ctx: Ctx, command: Cmd) -> Result {
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
            let env = ctx.quartz.env();
            let curr_env = env.current().unwrap();
            println!("{}", curr_env.name);
        }
        Cmd::Cookies(args) => action::cookie::print(ctx, args),
        Cmd::Endpoint => endpoint(ctx)?,
        Cmd::Snippet(args) => action::snippet::cmd(ctx, args)?,
    };

    Ok(())
}

pub fn url(ctx: Ctx) {
    let endpoint = ctx.quartz.endpoint();
    let handle = endpoint.current().unwrap();
    let endpoint = handle.endpoint().unwrap();
    println!("{}", endpoint.url);
}

pub fn method(ctx: Ctx) {
    let endpoint = ctx.quartz.endpoint();
    let handle = endpoint.current().unwrap();
    let endpoint = handle.endpoint().unwrap();
    println!("{}", endpoint.method);
}

pub fn handle(ctx: Ctx) {
    let endpoint = ctx.quartz.endpoint();
    let handle = endpoint.current().unwrap();
    println!("{}", handle.head());
}

pub fn endpoint(ctx: Ctx) -> Result {
    let endpoint = ctx.quartz.endpoint();
    let handle = endpoint.current().ok_or(Error::NoHandleInUse)?;
    let endpoint = handle.endpoint()?;

    println!("{}", endpoint.to_toml()?);
    Ok(())
}
