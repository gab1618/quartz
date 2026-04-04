use crate::Result;
use std::path::{Path, PathBuf};

use quartz_core::Quartz;

#[derive(clap::Args, Debug)]
pub struct Args {
    directory: Option<PathBuf>,
}

pub fn cmd(args: Args) -> Result {
    let directory = args.directory.unwrap_or(Path::new(".").to_path_buf());
    Quartz::init(directory)?;

    Ok(())
}
