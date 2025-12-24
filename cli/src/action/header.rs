use crate::{cli::HeaderCmd as Cmd, ctx::Ctx};
use quartz_core::error::QuartzResult;

pub fn cmd(ctx: Ctx, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Get { key } => get(ctx, key),
        Cmd::Set { name, value } => set(ctx, name, value),
        Cmd::Rm { key } => rm(ctx, key),
        Cmd::Ls => ls(ctx),
    }
}

pub fn get(ctx: Ctx, key: String) -> QuartzResult {
    let handle = ctx.quartz.handle().unwrap();
    let endpoint = handle.endpoint().unwrap();
    if let Some(header) = endpoint.headers.get(&key) {
        println!("{}", header);
    } else {
        panic!("no header named {} found", key);
    }

    Ok(())
}

pub fn set(ctx: Ctx, name: String, value: String) -> QuartzResult {
    let handle = ctx.quartz.handle().unwrap();
    let mut endpoint = handle.endpoint().unwrap();
    endpoint.headers.0.insert(name, value);
    handle.write_endpoint(&endpoint)?;
    Ok(())
}

pub fn rm(ctx: Ctx, keys: Vec<String>) -> QuartzResult {
    let handle = ctx.quartz.handle().unwrap();
    let mut endpoint = handle.endpoint().unwrap();

    for k in keys {
        if endpoint.headers.contains_key(&k) {
            endpoint.headers.remove(&k);
            println!("Removed header: {}", k);
        } else {
            eprintln!("{}: No such header", k);
        }
    }

    handle.write_endpoint(&endpoint)?;
    Ok(())
}

pub fn ls(ctx: Ctx) -> QuartzResult {
    let handle = ctx.quartz.handle().unwrap();
    let endpoint = handle.endpoint().unwrap();

    print!("{}", endpoint.headers);
    Ok(())
}
