use crate::{
    cli::QueryCmd as Cmd,
    ctx::Ctx,
    error::{Error, Result},
};
use colored::Colorize;
use quartz_core::pairmap::PairMap;

#[derive(clap::Args, Debug)]
pub struct GetArgs {
    key: String,
}

#[derive(clap::Args, Debug)]
pub struct SetArgs {
    #[arg(name = "QUERY", required = true)]
    queries: Vec<String>,
}

#[derive(clap::Args, Debug)]
pub struct RmArgs {
    #[arg(name = "QUERY", required = true)]
    keys: Vec<String>,
}

pub fn cmd(ctx: Ctx, command: Cmd) -> Result {
    match command {
        Cmd::Get(args) => get(ctx, args.key)?,
        Cmd::Set(args) => set(ctx, args.queries)?,
        Cmd::Rm(args) => rm(ctx, args.keys)?,
        Cmd::Ls => ls(ctx)?,
    };

    Ok(())
}

pub fn get(ctx: Ctx, key: String) -> Result {
    let handle = ctx.quartz.current().ok_or(Error::NoHandleInUse)?;
    let endpoint = handle.endpoint()?;

    let value = endpoint
        .query
        .get(&key)
        .unwrap_or_else(|| panic!("no query param {} found", key.red()));

    println!("{value}");
    Ok(())
}

pub fn set(ctx: Ctx, queries: Vec<String>) -> Result {
    let handle = ctx.quartz.current().unwrap();
    let mut endpoint = handle.endpoint().unwrap();

    for input in queries {
        endpoint.query.set(&input)?;
    }

    handle.write_endpoint(&endpoint)?;

    Ok(())
}

pub fn rm(ctx: Ctx, keys: Vec<String>) -> Result {
    let handle = ctx.quartz.current().unwrap();
    let mut endpoint = handle.endpoint()?;

    for k in keys {
        if endpoint.query.contains_key(&k) {
            endpoint.query.remove(&k);
            println!("Removed query param: {}", k);
        } else {
            eprintln!("{}: No such query param", k);
        }
    }

    handle.write_endpoint(&endpoint)?;
    Ok(())
}

pub fn ls(ctx: Ctx) -> Result {
    let handle = ctx.quartz.current().ok_or(Error::NoHandleInUse)?;
    let endpoint = handle.endpoint()?;
    print!("{}", endpoint.query);

    Ok(())
}

pub fn print(ctx: Ctx) -> Result {
    let handle = ctx.quartz.current().ok_or(Error::NoHandleInUse)?;
    let endpoint = handle.endpoint()?;
    println!("{}", endpoint.query_string());

    Ok(())
}
