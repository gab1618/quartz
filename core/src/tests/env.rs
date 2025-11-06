use crate::tests::utils::TestQuartz;

#[test]
fn get_default_env() {
    let test_quartz = TestQuartz::empty().unwrap();
    let curr_env = test_quartz.inner.current_env();

    assert_eq!(curr_env.name, "default".to_string());
}

#[test]
fn create_env() {
    let test_quartz = TestQuartz::empty().unwrap();
    test_quartz.inner.create_env("dev").unwrap();
    let envs = test_quartz.inner.get_envs().unwrap();
    let has_new_env = envs.into_iter().any(|entry| entry == "dev".to_string());
    assert!(has_new_env);

    test_quartz.inner.create_env("dev").unwrap_err();
}

#[test]
fn switch_env() {
    let test_quartz = TestQuartz::empty().unwrap();
    test_quartz.inner.create_env("dev").unwrap();
    test_quartz.inner.switch_env("dev").unwrap();
    test_quartz.inner.switch_env("default").unwrap();
    assert!(test_quartz.inner.switch_env("invalid").is_err());

    assert_eq!(test_quartz.inner.current_env().name, "default");
    test_quartz.inner.switch_env("dev").unwrap();
    assert_eq!(test_quartz.inner.current_env().name, "dev");
}

#[test]
fn remove_env() {
    let test_quartz = TestQuartz::empty().unwrap();
    test_quartz.inner.create_env("dev").unwrap();

    test_quartz.inner.remove_env("dev").unwrap();
    test_quartz.inner.create_env("dev").unwrap();

    test_quartz.inner.switch_env("dev").unwrap();
    test_quartz.inner.remove_env("dev").unwrap_err();
}

#[test]
fn add_header_to_env() {
    let test_quartz = TestQuartz::empty().unwrap();
    test_quartz.inner.env_header_set("Header1: value1").unwrap();
    test_quartz.inner.env_header_set("Header2: value2").unwrap();

    let found_header = test_quartz.inner.env_header_get("Header1").unwrap();
    assert_eq!(found_header, "value1".to_owned());

    test_quartz.inner.create_env("dev").unwrap();
    test_quartz.inner.switch_env("dev").unwrap();
    test_quartz.inner.env_header_get("Header1").unwrap_err();
    test_quartz.inner.switch_env("default").unwrap();

    test_quartz.inner.env_header_rm("Header1").unwrap();
    test_quartz.inner.env_header_get("Header1").unwrap_err();
}

#[test]
fn copy_env() {
    let test_quartz = TestQuartz::empty().unwrap();
    test_quartz.inner.create_env("dev").unwrap();
    test_quartz.inner.switch_env("dev").unwrap();

    test_quartz.inner.env_header_set("Header1: value1").unwrap();
    let retrieved_header = test_quartz.inner.env_header_get("Header1").unwrap();
    assert_eq!(&retrieved_header, "value1");

    test_quartz.inner.cp_env("dev", "dev-copy").unwrap();
    test_quartz.inner.switch_env("dev-copy").unwrap();
    let cloned_header = test_quartz.inner.env_header_get("Header1").unwrap();

    assert_eq!(retrieved_header, cloned_header);
}
