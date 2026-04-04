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
                action::query::get(ctx, key)?;
            } else {
                action::query::print(ctx)?;
            }
        }
        Cmd::Headers { key } => {
            if let Some(key) = key {
                action::header::get(ctx, key)?;
            } else {
                action::header::ls(ctx)?;
            }
        }
        Cmd::Url => url(ctx)?,
        Cmd::Method => method(ctx)?,
        Cmd::Body => action::body::print(ctx)?,
        Cmd::Handle => handle(ctx)?,
        Cmd::Env => {
            let curr_env = ctx.quartz.current_env()?;
            println!("{}", curr_env.name);
        }
        Cmd::Cookies(args) => action::cookie::print(ctx, args)?,
        Cmd::Endpoint => endpoint(ctx)?,
        Cmd::Snippet(args) => action::snippet::cmd(ctx, args)?,
    };

    Ok(())
}

pub fn url(ctx: Ctx) -> Result {
    let handle = ctx.quartz.current_endpoint().ok_or(Error::NoHandleInUse)?;
    let endpoint = handle.endpoint()?;
    println!("{}", endpoint.url);

    Ok(())
}

pub fn method(ctx: Ctx) -> Result {
    let handle = ctx.quartz.current_endpoint().ok_or(Error::NoHandleInUse)?;
    let endpoint = handle.endpoint()?;
    println!("{}", endpoint.method);

    Ok(())
}

pub fn handle(ctx: Ctx) -> Result {
    let handle = ctx.quartz.current_endpoint().ok_or(Error::NoHandleInUse)?;
    println!("{}", handle.head());

    Ok(())
}

pub fn endpoint(ctx: Ctx) -> Result {
    let handle = ctx.quartz.current_endpoint().ok_or(Error::NoHandleInUse)?;
    let endpoint = handle.endpoint()?;

    println!("{}", endpoint.to_toml()?);
    Ok(())
}
