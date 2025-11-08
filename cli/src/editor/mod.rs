use quartz_core::{Quartz, QuartzError, QuartzResult, editor::Editor, pager::Pager};

#[derive(Default)]
pub struct FileEditor {}

impl Editor for FileEditor {
    fn edit<E: Editor, P: Pager>(
        &self,
        quartz: &Quartz<E, P>,
        file_path: &std::path::PathBuf,
    ) -> QuartzResult {
        let editor = self.editor(quartz)?;
        let _ = std::process::Command::new(editor)
            .arg(file_path)
            .status()
            .map_err(|_| QuartzError::Internal);

        Ok(())
    }
}
