use std::ops::{Deref, DerefMut};

use tempfile::{TempDir, tempdir};

use crate::Quartz;

pub struct TestQuartz {
    pub inner: Quartz,
    #[allow(unused)]
    dir: TempDir,
    #[allow(unused)]
    config_dir: TempDir,
}

impl TestQuartz {
    pub fn empty() -> Self {
        let dir = tempdir().unwrap();
        let config_dir = tempdir().unwrap();

        let dir_buf_path = dir.path().to_path_buf();
        let qz = Quartz::init(dir_buf_path).unwrap();

        Self {
            inner: qz,
            dir,
            config_dir,
        }
    }
}

impl Deref for TestQuartz {
    type Target = Quartz;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TestQuartz {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
