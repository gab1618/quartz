use crate::QuartzResult;
use std::path::{Path, PathBuf};

use quartz_core::{Quartz, error::QuartzError};

#[derive(clap::Args, Debug)]
pub struct Args {
    directory: Option<PathBuf>,
}

pub fn cmd(args: Args) -> QuartzResult {
    let directory = args.directory.unwrap_or(Path::new(".").to_path_buf());
    let home_dir = std::env::home_dir().ok_or(QuartzError::Internal)?;
    Quartz::init(directory, home_dir)?;

    Ok(())
}
