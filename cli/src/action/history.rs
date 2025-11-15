use std::{io::Write, process::Stdio};

use quartz_core::{Quartz, QuartzError, QuartzResult};

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Maximum number of requests to be listed
    #[arg(short = 'n', long, value_name = "N")]
    max_count: Option<usize>,
}

pub fn cmd(quartz: Quartz, args: Args) -> QuartzResult {
    let history = quartz.history()?;
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

    let pager = quartz.ctx.config.parse().preferences.pager();

    let mut child = std::process::Command::new(&pager)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap_or_else(|err| {
            panic!("failed to open pager: {}\n\n{}", pager, err);
        });

    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(output.as_bytes())
        .map_err(|_| QuartzError::Internal)?;
    child.wait().map_err(|_| QuartzError::Internal)?;

    Ok(())
}
