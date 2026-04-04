use tempfile::tempdir;

use crate::Quartz;

#[test]
fn initialize_quartz() {
    let base_dir = tempdir().unwrap();
    let base_dir_path = base_dir.path().to_path_buf();

    let config_dir = tempdir().unwrap();
    let config_dir_path = config_dir.path().to_path_buf();

    Quartz::init(base_dir_path, config_dir_path).unwrap();
}

#[test]
fn initialize_twice() {
    let base_dir = tempdir().unwrap();
    let base_dir_path = base_dir.path().to_path_buf();

    let config_dir = tempdir().unwrap();
    let config_dir_path = config_dir.path().to_path_buf();

    Quartz::init(base_dir_path.clone(), config_dir_path.clone()).unwrap();
    assert!(Quartz::init(base_dir_path, config_dir_path).is_err());
}
