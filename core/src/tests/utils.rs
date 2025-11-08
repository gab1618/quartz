use std::{
    cell::Cell,
    ops::{Deref, DerefMut},
};

use tempfile::{TempDir, tempdir};

use crate::{Quartz, QuartzError, QuartzResult, editor::Editor, pager::Pager};

#[derive(Default)]
pub struct MockEditor {
    edit_calls_count: Cell<u8>,
}
impl Editor for MockEditor {
    fn edit<E: Editor, P: Pager>(
        &self,
        _quartz: &Quartz<E, P>,
        _file_path: &std::path::PathBuf,
    ) -> QuartzResult {
        let curr_call_count = self.edit_calls_count.take();
        self.edit_calls_count.set(curr_call_count + 1);

        Ok(())
    }
}

#[derive(Default)]
pub struct MockPager {}
impl Pager for MockPager {
    fn paginate<E: Editor, P: Pager>(
        &self,
        _quartz: &Quartz<E, P>,
        _content: &[u8],
    ) -> QuartzResult {
        Ok(())
    }
}

pub struct TestQuartz {
    pub inner: Quartz<MockEditor, MockPager>,
    #[allow(unused)]
    dir: TempDir,
    #[allow(unused)]
    config_dir: TempDir,
}

impl TestQuartz {
    pub fn empty() -> QuartzResult<Self> {
        let dir = tempdir().map_err(|_| QuartzError::Internal)?;
        let config_dir = tempdir().map_err(|_| QuartzError::Internal)?;

        let dir_buf_path = dir.path().to_path_buf();
        let config_dir_path = config_dir.path().to_path_buf();
        let mock_editor = MockEditor::default();
        let mock_pager = MockPager::default();
        let qz = Quartz::init(dir_buf_path, config_dir_path, mock_editor, mock_pager)
            .map_err(|_| QuartzError::Internal)?;

        Ok(Self {
            inner: qz,
            dir,
            config_dir,
        })
    }
}

impl Deref for TestQuartz {
    type Target = Quartz<MockEditor, MockPager>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TestQuartz {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
