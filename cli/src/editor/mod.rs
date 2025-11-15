use quartz_core::{Quartz, QuartzError, QuartzResult, editor::Editor};

#[derive(Default)]
pub struct CliEditor {}

impl Editor for CliEditor {
    fn edit<E: Editor>(&self, quartz: &Quartz<E>, file_path: &std::path::PathBuf) -> QuartzResult {
        let editor = self.editor(quartz)?;
        let _ = std::process::Command::new(editor)
            .arg(file_path)
            .status()
            .map_err(|_| QuartzError::Internal);

        Ok(())
    }
}
