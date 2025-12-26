use crate::{cli::HeaderCmd as Cmd, ctx::Ctx};
use quartz_core::error::Result;

pub fn cmd(ctx: Ctx, command: Cmd) -> Result {
    match command {
        Cmd::Get { key } => get(ctx, key),
        Cmd::Set { name, value } => set(ctx, name, value),
        Cmd::Rm { key } => rm(ctx, key),
        Cmd::Ls => ls(ctx),
    }
}

pub fn get(ctx: Ctx, key: String) -> Result {
    let endpoint = ctx.quartz.endpoint();
    let handle = endpoint.current().unwrap();
    let endpoint = handle.endpoint().unwrap();
    if let Some(header) = endpoint.headers.get(&key) {
        println!("{}", header);
    } else {
        panic!("no header named {} found", key);
    }

    Ok(())
}

pub fn set(ctx: Ctx, name: String, value: String) -> Result {
    let endpoint = ctx.quartz.endpoint();
    let handle = endpoint.current().unwrap();
    let mut endpoint = handle.endpoint().unwrap();
    endpoint.headers.0.insert(name, value);
    handle.write_endpoint(&endpoint)?;
    Ok(())
}

pub fn rm(ctx: Ctx, keys: Vec<String>) -> Result {
    let endpoint = ctx.quartz.endpoint();
    let handle = endpoint.current().unwrap();
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

pub fn ls(ctx: Ctx) -> Result {
    let endpoint = ctx.quartz.endpoint();
    let handle = endpoint.current().unwrap();
    let endpoint = handle.endpoint().unwrap();

    print!("{}", endpoint.headers);
    Ok(())
}
