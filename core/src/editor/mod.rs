use std::path::PathBuf;

use crate::{Quartz, QuartzResult};

pub trait Editor {
    fn edit<E: Editor>(&self, quartz: &Quartz<E>, file_path: &PathBuf) -> QuartzResult;
}
