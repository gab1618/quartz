use tempfile::tempdir;

use crate::Quartz;

#[test]
fn initialize_quartz() {
    let base_dir = tempdir().unwrap();
    let base_dir_path = base_dir.path().to_path_buf();
    Quartz::init(&base_dir_path).unwrap();
}

#[test]
fn initialize_twice() {
    let base_dir = tempdir().unwrap();
    let base_dir_path = base_dir.path().to_path_buf();

    Quartz::init(&base_dir_path).unwrap();
    assert!(Quartz::init(&base_dir_path).is_err());
}

#[test]
fn detect_git() {
    let base_dir = tempdir().unwrap();
    let base_dir_path = base_dir.path().to_path_buf();

    std::fs::create_dir_all(base_dir_path.join(".git")).unwrap();
    Quartz::init(&base_dir_path).unwrap();

    let gitignore_path = base_dir_path.join(".gitignore");
    assert!(gitignore_path.exists());
}
