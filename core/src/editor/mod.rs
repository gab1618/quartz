use std::path::PathBuf;

use crate::{Quartz, QuartzResult, pager::Pager};

pub trait Editor {
    fn edit<E: Editor, P: Pager>(&self, quartz: &Quartz<E, P>, file_path: &PathBuf)
    -> QuartzResult;
    fn editor<E: Editor, P: Pager>(&self, quartz: &Quartz<E, P>) -> QuartzResult<String> {
        let current_editor = quartz.config_get("preferences.editor")?;

        Ok(current_editor)
    }
}
