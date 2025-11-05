use crate::tests::utils::TestQuartz;

#[test]
fn save_and_retrieve_config() {
    let quartz = TestQuartz::empty().unwrap();

    quartz.config_set("preferences.editor", "nvim").unwrap();
    quartz
        .config_set("preferences.editor-invalid", "nvim")
        .unwrap_err();

    let retrieved = quartz.config_get("preferences.editor").unwrap();
    assert_eq!(retrieved, "nvim");
    quartz.config_get("preferences.editor-invalid").unwrap_err();
}
