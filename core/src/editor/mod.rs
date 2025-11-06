use std::path::PathBuf;

use crate::{Quartz, QuartzResult};

pub trait Editor {
    fn edit<E: Editor>(&self, quartz: &Quartz<E>, file_path: &PathBuf) -> QuartzResult;
    fn editor<E: Editor>(&self, quartz: &Quartz<E>) -> QuartzResult<String> {
        let current_editor = quartz.config_get("preferences.editor")?;

        Ok(current_editor)
    }
}
