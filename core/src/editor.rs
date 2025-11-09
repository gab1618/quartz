use std::path::PathBuf;

use crate::{Quartz, QuartzResult, pager::Pager};

pub trait Editor {
    fn edit<E: Editor, P: Pager>(&self, quartz: &Quartz<E, P>, file_path: &PathBuf)
    -> QuartzResult;
    fn editor<E: Editor, P: Pager>(&self, quartz: &Quartz<E, P>) -> QuartzResult<String> {
        let config = quartz.config();
        let current_editor = config.get("preferences.editor")?;

        Ok(current_editor)
    }
}

#[derive(Default)]
pub struct NoEditor {}
impl Editor for NoEditor {
    fn edit<E: Editor, P: Pager>(
        &self,
        _quartz: &Quartz<E, P>,
        _file_path: &PathBuf,
    ) -> QuartzResult {
        Ok(())
    }
}
