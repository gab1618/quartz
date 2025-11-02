use std::ops::Deref;

use tempfile::{TempDir, tempdir};

use crate::Quartz;

pub struct TestQuartz {
    pub inner: Quartz,
    dir: TempDir,
}

impl TestQuartz {
    pub fn empty() -> Self {
        // TODO: add proper error handling
        let dir = tempdir().unwrap();
        let dir_buf_path = dir.path().to_path_buf();
        let qz = Quartz::init(&dir_buf_path).unwrap();

        Self { inner: qz, dir }
    }
}

impl Deref for TestQuartz {
    type Target = Quartz;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
