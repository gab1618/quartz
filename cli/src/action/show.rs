use crate::{action, cli::ShowCmd as Cmd};
use quartz_core::{
    Quartz, QuartzError, QuartzResult,
};

pub fn cmd(quartz: Quartz, command: Cmd) -> QuartzResult {
    match command {
        Cmd::Query { key } => {
            if let Some(key) = key {
                action::query::get(quartz, key);
            } else {
                action::query::print(quartz);
            }
        }
        Cmd::Headers { key } => {
            if let Some(key) = key {
                action::header::get(quartz, key)?;
            } else {
                action::header::ls(quartz)?;
            }
        }
        Cmd::Url => url(quartz),
        Cmd::Method => method(quartz),
        Cmd::Body => action::body::print(quartz)?,
        Cmd::Handle => handle(quartz),
        Cmd::Env => {
            let curr_env = quartz.current_env().unwrap();
            println!("{}", curr_env.name);
        }
        Cmd::Cookies(args) => action::cookie::print(quartz, args),
        Cmd::Endpoint => endpoint(quartz)?,
        Cmd::Snippet(args) => action::snippet::cmd(quartz, args)?,
    };

    Ok(())
}

pub fn url(quartz: Quartz) {
    let handle = quartz.current_handle().unwrap();
    let endpoint = handle.endpoint(&quartz).unwrap();
    println!("{}", endpoint.url);
}

pub fn method(quartz: Quartz) {
    let handle = quartz.current_handle().unwrap();
    let endpoint = handle.endpoint(&quartz).unwrap();
    println!("{}", endpoint.method);
}

pub fn handle(quartz: Quartz) {
    let handle = quartz.current_handle().unwrap();
    println!("{}", handle.head());
}

pub fn endpoint(quartz: Quartz) -> QuartzResult {
    let handle = quartz.current_handle().unwrap();
    let endpoint = handle.endpoint(&quartz).unwrap();

    println!("{}", endpoint.to_toml().map_err(|_| QuartzError::Internal)?);
    Ok(())
}
