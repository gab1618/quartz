use crate::{cli::QueryCmd as Cmd, ctx::Ctx};
use colored::Colorize;
use quartz_core::{error::QuartzResult, pairmap::PairMap};

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

pub fn cmd(ctx: Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Get(args) => get(ctx, args.key),
        Cmd::Set(args) => set(ctx, args.queries)?,
        Cmd::Rm(args) => rm(ctx, args.keys)?,
        Cmd::Ls => ls(ctx),
    };

    Ok(())
}

pub fn get(ctx: Ctx, key: String) {
    let handle = ctx.quartz.handle().unwrap();
    let endpoint = handle.endpoint(&ctx.quartz).unwrap();

    let value = endpoint
        .query
        .get(&key)
        .unwrap_or_else(|| panic!("no query param {} found", key.red()));

    println!("{value}");
}

pub fn set(ctx: Ctx, queries: Vec<String>) -> QuartzResult {
    let handle = ctx.quartz.handle().unwrap();
    let mut endpoint = handle.endpoint(&ctx.quartz).unwrap();

    for input in queries {
        endpoint.query.set(&input)?;
    }

    endpoint.write()?;

    Ok(())
}

pub fn rm(ctx: Ctx, keys: Vec<String>) -> QuartzResult {
    let handle = ctx.quartz.handle().unwrap();
    let mut endpoint = handle.endpoint(&ctx.quartz).unwrap();

    for k in keys {
        if endpoint.query.contains_key(&k) {
            endpoint.query.remove(&k);
            println!("Removed query param: {}", k);
        } else {
            eprintln!("{}: No such query param", k);
        }
    }

    endpoint.write()?;
    Ok(())
}

pub fn ls(ctx: Ctx) {
    let handle = ctx.quartz.handle().unwrap();
    let endpoint = handle.endpoint(&ctx.quartz).unwrap();
    print!("{}", endpoint.query);
}

pub fn print(ctx: Ctx) {
    let handle = ctx.quartz.handle().unwrap();
    let endpoint = handle.endpoint(&ctx.quartz).unwrap();
    println!("{}", endpoint.query_string());
}
