use quartz_core::{Quartz, QuartzError, QuartzResult, editor::Editor};

#[derive(Default)]
pub struct FileEditor {}

impl Editor for FileEditor {
    fn edit<E: Editor>(&self, quartz: &Quartz<E>, file_path: &std::path::PathBuf) -> QuartzResult {
        let editor = quartz.config_get("preferences.editor")?;
        let _ = std::process::Command::new(editor)
            .arg(file_path)
            .status()
            .map_err(|_| QuartzError::Internal);

        Ok(())
    }
}
