use crate::cli::HeaderCmd as Cmd;
use quartz_core::{Quartz, QuartzResult, pairmap::PairMap};

pub fn cmd(quartz: Quartz, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Get { key } => get(quartz, key),
        Cmd::Set { header } => set(quartz, header),
        Cmd::Rm { key } => rm(quartz, key),
        Cmd::Ls => ls(quartz),
    }
}

pub fn get(quartz: Quartz, key: String) -> QuartzResult {
    let handle = quartz.current_handle().unwrap();
    let endpoint = handle.endpoint(&quartz).unwrap();
    if let Some(header) = endpoint.headers.get(&key) {
        println!("{}", header);
    } else {
        panic!("no header named {} found", key);
    }

    Ok(())
}

pub fn set(quartz: Quartz, header: String) -> QuartzResult {
    let handle = quartz.current_handle().unwrap();
    let mut endpoint = handle.endpoint(&quartz).unwrap();
    endpoint.headers.set(&header)?;
    endpoint.write();
    Ok(())
}

pub fn rm(quartz: Quartz, keys: Vec<String>) -> QuartzResult {
    let handle = quartz.current_handle().unwrap();
    let mut endpoint = handle.endpoint(&quartz).unwrap();

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

pub fn ls(quartz: Quartz) -> QuartzResult {
    let handle = quartz.current_handle().unwrap();
    let endpoint = handle.endpoint(&quartz).unwrap();

    print!("{}", endpoint.headers);
    Ok(())
}
