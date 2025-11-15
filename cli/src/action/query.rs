use crate::cli::QueryCmd as Cmd;
use colored::Colorize;
use quartz_core::{Quartz, QuartzResult, pairmap::PairMap};

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

pub fn cmd(quartz: Quartz, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Get(args) => get(quartz, args.key),
        Cmd::Set(args) => set(quartz, args.queries),
        Cmd::Rm(args) => rm(quartz, args.keys)?,
        Cmd::Ls => ls(quartz),
    };

    Ok(())
}

pub fn get(quartz: Quartz, key: String) {
    let handle = quartz.current_handle().unwrap();
    let endpoint = handle.endpoint(&quartz).unwrap();

    let value = endpoint
        .query
        .get(&key)
        .unwrap_or_else(|| panic!("no query param {} found", key.red()));

    println!("{value}");
}

pub fn set(quartz: Quartz, queries: Vec<String>) {
    let handle = quartz.current_handle().unwrap();
    let mut endpoint = handle.endpoint(&quartz).unwrap();

    for input in queries {
        endpoint.query.set(&input).unwrap();
    }

    endpoint.write();
}

pub fn rm(quartz: Quartz, keys: Vec<String>) -> QuartzResult {
    let handle = quartz.current_handle().unwrap();
    let mut endpoint = handle.endpoint(&quartz).unwrap();

    for k in keys {
        if endpoint.query.contains_key(&k) {
            endpoint.query.remove(&k);
            println!("Removed query param: {}", k);
        } else {
            eprintln!("{}: No such query param", k);
        }
    }

    endpoint.write();
    Ok(())
}

pub fn ls(quartz: Quartz) {
    let handle = quartz.current_handle().unwrap();
    let endpoint = handle.endpoint(&quartz).unwrap();
    print!("{}", endpoint.query);
}

pub fn print(quartz: Quartz) {
    let handle = quartz.current_handle().unwrap();
    let endpoint = handle.endpoint(&quartz).unwrap();
    println!("{}", endpoint.query_string());
}
