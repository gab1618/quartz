use crate::{Error, Result};
use crate::{
    cli::{LastCmd as Cmd, LastResCmd as ResCmd},
    ctx::Ctx,
};
use quartz_core::history::{self};

pub fn cmd(ctx: Ctx, maybe_command: Option<Cmd>) -> Result {
    let entry = ctx
        .quartz
        .last_history_entry()?
        .ok_or(Error::NoHistoryEntry)?;

    if maybe_command.is_none() {
        println!("{entry}");
        return Ok(());
    }

    if let Some(command) = maybe_command {
        match command {
            Cmd::Handle => println!("{}", entry.handle()),
            Cmd::Req => req(&entry),
            Cmd::Res { command } => res(command, &entry),
        }
    };

    Ok(())
}

pub fn req(entry: &history::Entry) {
    req_head(entry);
}

pub fn req_head(entry: &history::Entry) {
    let iter = entry
        .messages()
        .iter()
        .filter_map(|p| match p.starts_with('>') {
            true => Some(
                p.split('\n')
                    .map(|s| s.trim_start_matches('>').trim())
                    .collect::<Vec<&str>>()
                    .join("\n"),
            ),
            false => None,
        });

    for m in iter {
        println!("{m}");
    }
}

pub fn res(command: Option<ResCmd>, entry: &history::Entry) {
    if let Some(command) = command {
        match command {
            ResCmd::Head => res_head(entry),
            ResCmd::Body => res_body(entry),
        }
    } else {
        res_head(entry);
        res_body(entry);
    }
}

pub fn res_head(entry: &history::Entry) {
    let iter = entry
        .messages()
        .iter()
        .filter_map(|p| match p.starts_with('<') {
            true => Some(
                p.split('\n')
                    .map(|s| s.trim_start_matches('<').trim())
                    .collect::<Vec<&str>>()
                    .join("\n"),
            ),
            false => None,
        });

    for m in iter {
        println!("{m}");
    }
}

pub fn res_body(entry: &history::Entry) {
    if let Some(body) = entry.messages().last() {
        println!("{}", body);
    }
}
