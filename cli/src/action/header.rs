use crate::{cli::HeaderCmd as Cmd, ctx::Ctx};
use quartz_core::{error::QuartzResult, pairmap::PairMap};

pub fn cmd(ctx: Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Get { key } => get(ctx, key),
        Cmd::Set { header } => set(ctx, header),
        Cmd::Rm { key } => rm(ctx, key),
        Cmd::Ls => ls(ctx),
    }
}

pub fn get(ctx: Ctx, key: String) -> QuartzResult {
    let handle = ctx.quartz.handle().unwrap();
    let endpoint = handle.endpoint(&ctx.quartz).unwrap();
    if let Some(header) = endpoint.headers.get(&key) {
        println!("{}", header);
    } else {
        panic!("no header named {} found", key);
    }

    Ok(())
}

pub fn set(ctx: Ctx, header: String) -> QuartzResult {
    let handle = ctx.quartz.handle().unwrap();
    let mut endpoint = handle.endpoint(&ctx.quartz).unwrap();
    endpoint.headers.set(&header)?;
    endpoint.write();
    Ok(())
}

pub fn rm(ctx: Ctx, keys: Vec<String>) -> QuartzResult {
    let handle = ctx.quartz.handle().unwrap();
    let mut endpoint = handle.endpoint(&ctx.quartz).unwrap();

    for k in keys {
        if endpoint.headers.contains_key(&k) {
            endpoint.headers.remove(&k);
            println!("Removed header: {}", k);
        } else {
            eprintln!("{}: No such header", k);
        }
    }

    endpoint.write();
    Ok(())
}

pub fn ls(ctx: Ctx) -> QuartzResult {
    let handle = ctx.quartz.handle().unwrap();
    let endpoint = handle.endpoint(&ctx.quartz).unwrap();

    print!("{}", endpoint.headers);
    Ok(())
}
