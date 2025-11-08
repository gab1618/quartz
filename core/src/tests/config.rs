use crate::tests::utils::TestQuartz;

#[test]
fn save_and_retrieve_config() {
    let quartz = TestQuartz::empty().unwrap();
    let config = quartz.config();

    config.set("preferences.editor", "nvim").unwrap();
    config
        .set("preferences.editor-invalid", "nvim")
        .unwrap_err();

    let retrieved = config.get("preferences.editor").unwrap();
    assert_eq!(retrieved, "nvim");
    config.get("preferences.editor-invalid").unwrap_err();
}
