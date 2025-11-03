use std::path::PathBuf;

use crate::QuartzResult;

pub trait Editor {
    fn edit(&self, file_path: &PathBuf) -> QuartzResult;
}
