use quartz_core::{QuartzError, QuartzResult, editor::Editor};

#[derive(Default)]
pub struct FileEditor {
    editor: String,
}

impl FileEditor {
    pub fn new(editor: String) -> Self {
        Self { editor }
    }
}

impl Editor for FileEditor {
    fn edit(&self, file_path: &std::path::PathBuf) -> QuartzResult {
        let _ = std::process::Command::new(&self.editor)
            .arg(file_path)
            .status()
            .map_err(|_| QuartzError::Internal);

        Ok(())
    }
}
