use crate::tests::utils::TestQuartz;

#[test]
fn get_default_env() {
    let test_quartz = TestQuartz::empty();
    let env = test_quartz.env();
    let curr_env = env.current().unwrap();

    assert_eq!(curr_env.name, "default".to_string());
}

#[test]
fn create_env() {
    let test_quartz = TestQuartz::empty();
    let env = test_quartz.env();
    env.create("dev".into()).unwrap();
    let mut envs = env.envs().unwrap();
    let has_new_env = envs.any(|entry| {
        entry
            .map(|entry| entry == "dev".to_string())
            .unwrap_or(false)
    });
    assert!(has_new_env);
}

#[test]
fn switch_env() {
    let test_quartz = TestQuartz::empty();
    let env = test_quartz.env();
    env.create("dev".into()).unwrap();
    env.switch("dev".into()).unwrap();
    env.switch("default".into()).unwrap();
    assert!(env.switch("invalid".into()).is_err());

    assert_eq!(env.current().unwrap().name, "default");
    env.switch("dev".into()).unwrap();
    assert_eq!(env.current().unwrap().name, "dev");
}

#[test]
fn remove_env() {
    let test_quartz = TestQuartz::empty();
    let env = test_quartz.env();
    env.create("dev".into()).unwrap();

    env.remove("dev".into()).unwrap();
    env.create("dev".into()).unwrap();

    env.switch("dev".into()).unwrap();
    env.remove("dev".into()).unwrap_err();
}

#[test]
fn add_header_to_env() {
    let test_quartz = TestQuartz::empty();
    let env = test_quartz.env();
    let curr_env = env.current().unwrap();
    let mut curr_env_value = curr_env.read().unwrap();
    curr_env_value
        .header_set("Header1".to_owned(), "value1".to_owned())
        .unwrap();
    curr_env_value
        .header_set("Header2".to_owned(), "value2".to_owned())
        .unwrap();

    let found_header = curr_env_value.header_get("Header1").unwrap();
    assert_eq!(found_header, "value1");

    env.create("dev".into()).unwrap();
    let curr_env = env.switch("dev".into()).unwrap();
    let curr_env_value = curr_env.read().unwrap();
    assert!(curr_env_value.header_get("Header1").is_none());

    let curr_env = env.switch("default".into()).unwrap();
    let mut curr_env_value = curr_env.read().unwrap();

    curr_env_value.header_rm("Header1").unwrap();
    assert!(curr_env_value.header_get("Header1").is_none());
}

#[test]
fn copy_env() {
    let test_quartz = TestQuartz::empty();
    let env = test_quartz.env();
    env.create("dev".into()).unwrap();
    env.switch("dev".into()).unwrap();

    let curr_env = env.current().unwrap();
    let mut curr_env_value = curr_env.read().unwrap();
    curr_env_value
        .header_set("Header1".to_owned(), "value1".to_owned())
        .unwrap();
    curr_env.save(&curr_env_value).unwrap();
    let retrieved_header = curr_env_value.header_get("Header1").unwrap();
    assert_eq!(retrieved_header, "value1");

    let new_env = env.copy("dev".into(), "dev-copy".into()).unwrap();
    let new_env_value = new_env.read().unwrap();
    let cloned_header = new_env_value.header_get("Header1").unwrap();
    assert_eq!(retrieved_header, cloned_header);

    let curr_env = env.switch("dev-copy".into()).unwrap();
    let curr_env_value = curr_env.read().unwrap();
    let cloned_header = curr_env_value.header_get("Header1").unwrap();
    assert_eq!(retrieved_header, cloned_header);
}
