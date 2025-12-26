use crate::tests::utils::TestQuartz;

#[test]
fn get_default_env() {
    let test_quartz = TestQuartz::empty();
    let env = test_quartz.env();
    let curr_env = env.env().unwrap();

    assert_eq!(curr_env.name, "default".to_string());
}

#[test]
fn create_env() {
    let test_quartz = TestQuartz::empty();
    let env = test_quartz.env();
    env.create_env("dev".into()).unwrap();
    let mut envs = env.get_envs().unwrap();
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
    env.create_env("dev".into()).unwrap();
    env.switch_env("dev".into()).unwrap();
    env.switch_env("default".into()).unwrap();
    assert!(env.switch_env("invalid".into()).is_err());

    assert_eq!(env.env().unwrap().name, "default");
    env.switch_env("dev".into()).unwrap();
    assert_eq!(env.env().unwrap().name, "dev");
}

#[test]
fn remove_env() {
    let test_quartz = TestQuartz::empty();
    let env = test_quartz.env();
    env.create_env("dev".into()).unwrap();

    env.remove_env("dev".into()).unwrap();
    env.create_env("dev".into()).unwrap();

    env.switch_env("dev".into()).unwrap();
    env.remove_env("dev".into()).unwrap_err();
}

#[test]
fn add_header_to_env() {
    let test_quartz = TestQuartz::empty();
    let env = test_quartz.env();
    let mut curr_env = env.env().unwrap();
    curr_env
        .header_set("Header1".to_owned(), "value1".to_owned())
        .unwrap();
    curr_env
        .header_set("Header2".to_owned(), "value2".to_owned())
        .unwrap();

    let found_header = curr_env.header_get("Header1").unwrap();
    assert_eq!(found_header, "value1".to_owned());

    env.create_env("dev".into()).unwrap();
    let curr_env = env.switch_env("dev".into()).unwrap();
    curr_env.header_get("Header1").unwrap_err();
    let mut curr_env = env.switch_env("default".into()).unwrap();

    curr_env.header_rm("Header1").unwrap();
    curr_env.header_get("Header1").unwrap_err();
}

#[test]
fn copy_env() {
    let test_quartz = TestQuartz::empty();
    let env = test_quartz.env();
    env.create_env("dev".into()).unwrap();
    env.switch_env("dev".into()).unwrap();

    let mut curr_env = env.env().unwrap();
    curr_env
        .header_set("Header1".to_owned(), "value1".to_owned())
        .unwrap();
    curr_env.save().unwrap();
    let retrieved_header = curr_env.header_get("Header1").unwrap();
    assert_eq!(&retrieved_header, "value1");

    let new_env = env.cp_env("dev".into(), "dev-copy".into()).unwrap();
    let cloned_header = new_env.header_get("Header1").unwrap();
    assert_eq!(retrieved_header, cloned_header);

    let curr_env = env.switch_env("dev-copy".into()).unwrap();
    let cloned_header = curr_env.header_get("Header1").unwrap();
    assert_eq!(retrieved_header, cloned_header);
}
