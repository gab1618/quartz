use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
};

use tempfile::{TempDir, tempdir};

use crate::{Quartz, QuartzError, QuartzResult, editor::Editor};

#[derive(Default)]
pub struct MockEditor {
    edit_calls_count: Cell<u8>,
}
impl Editor for MockEditor {
    fn edit(&self, _file_path: &std::path::PathBuf) -> QuartzResult {
        let curr_call_count = self.edit_calls_count.take();
        self.edit_calls_count.set(curr_call_count + 1);

        Ok(())
    }
}

pub struct TestQuartz<E: Editor> {
    pub inner: Quartz<E>,
    #[allow(unused)]
    dir: TempDir,
    #[allow(unused)]
    config_dir: TempDir,
}

impl TestQuartz<MockEditor> {
    pub fn empty() -> QuartzResult<Self> {
        let dir = tempdir().map_err(|_| QuartzError::Internal)?;
        let config_dir = tempdir().map_err(|_| QuartzError::Internal)?;

        let dir_buf_path = dir.path().to_path_buf();
        let config_dir_path = config_dir.path().to_path_buf();
        let mock_editor = MockEditor::default();
        let qz = Quartz::init(&dir_buf_path, config_dir_path, mock_editor)
            .map_err(|_| QuartzError::Internal)?;

        Ok(Self {
            inner: qz,
            dir,
            config_dir,
        })
    }
}

impl<E: Editor> Deref for TestQuartz<E> {
    type Target = Quartz<E>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<E: Editor> DerefMut for TestQuartz<E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
