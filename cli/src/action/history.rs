use std::{io::Write, process::Stdio};

use crate::{Error, Result};

use crate::ctx::Ctx;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Maximum number of requests to be listed
    #[arg(short = 'n', long, value_name = "N")]
    max_count: Option<usize>,
}

pub fn cmd(ctx: Ctx, args: Args) -> Result {
    let history = ctx.quartz.history();
    let mut count = 0;
    let max_count = args.max_count.unwrap_or(usize::MAX);

    let mut output = String::new();
    for entry in history.entries()? {
        if count >= max_count {
            break;
        }

        count += 1;
        if count != 1 {
            // Separation between two entries
            output.push('\n');
        }

        output.push_str(&format!("{entry}\n"));
    }

    let pager = ctx.quartz.config().parse().preferences.pager();

    let mut child = std::process::Command::new(&pager)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(Error::SpawnPager)?;

    child
        .stdin
        .as_mut()
        .ok_or(Error::WriteStdin)?
        .write_all(output.as_bytes())
        .map_err(|_| Error::WriteStdin)?;
    child.wait().map_err(|_| Error::WriteStdin)?;

    Ok(())
}
