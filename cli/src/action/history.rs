use quartz_core::{Quartz, QuartzResult};

use crate::editor::FileEditor;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Maximum number of requests to be listed
    #[arg(short = 'n', long, value_name = "N")]
    max_count: Option<usize>,
}

pub fn cmd(quartz: Quartz<FileEditor>, args: Args) -> QuartzResult {
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

    quartz.paginate(output.as_bytes())?;

    Ok(())
}
