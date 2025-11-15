use std::path::PathBuf;

use crate::{Quartz, QuartzResult};

pub trait Editor {
    fn edit<E: Editor>(&self, quartz: &Quartz<E>, file_path: &PathBuf) -> QuartzResult;
    fn editor<E: Editor>(&self, quartz: &Quartz<E>) -> QuartzResult<String> {
        let config = quartz.config();
        let current_editor = config.get("preferences.editor")?;

        Ok(current_editor)
    }
}

#[derive(Default)]
pub struct NoEditor {}
impl Editor for NoEditor {
    fn edit<E: Editor>(&self, _quartz: &Quartz<E>, _file_path: &PathBuf) -> QuartzResult {
        Ok(())
    }
}
