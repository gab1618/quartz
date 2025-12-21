use crate::{endpoint::EndpointHandle, tests::utils::TestQuartz};

#[test]
fn test_simple_cp_handle() {
    let quartz = TestQuartz::empty();
    quartz.endpoint_create("testing").unwrap();
    quartz.handle_cp(false, "testing", "testing-copy").unwrap();

    let found_handle = quartz.new_handle("testing-copy");
    assert!(found_handle.exists());
}

#[test]
fn test_recursive_cp_handle() {
    let quartz = TestQuartz::empty();
    quartz.endpoint_create("testing").unwrap();
    quartz.endpoint_create("testing/ex").unwrap();
    quartz.endpoint_create("testing/ex2").unwrap();
    quartz.endpoint_create("testing/ex2/sub").unwrap();
    quartz.handle_cp(true, "testing", "testing-copy").unwrap();

    assert!(quartz.new_handle("testing-copy").exists());
    assert!(quartz.new_handle("testing-copy/ex").exists());
    assert!(quartz.new_handle("testing-copy/ex2").exists());
    assert!(quartz.new_handle("testing-copy/ex2/sub").exists());
}

#[test]
fn test_non_recursive_cp_handle() {
    let quartz = TestQuartz::empty();
    quartz.endpoint_create("testing").unwrap();
    quartz.endpoint_create("testing/ex").unwrap();
    quartz.endpoint_create("testing/ex2").unwrap();
    quartz.endpoint_create("testing/ex2/sub").unwrap();
    quartz.handle_cp(false, "testing", "testing-copy").unwrap();

    assert!(quartz.new_handle("testing-copy").exists());
    assert!(!quartz.new_handle("testing-copy/ex").exists());
    assert!(!quartz.new_handle("testing-copy/ex2").exists());
    assert!(!quartz.new_handle("testing-copy/ex2/sub").exists());
}

#[test]
fn test_cp_endpoint_spec() {
    let quartz = TestQuartz::empty();
    quartz.endpoint_create("testing").unwrap();
    quartz.endpoint_create("testing/ex2").unwrap();
    let created_handle = quartz.endpoint_create("testing/ex2/sub").unwrap();
    let mut created_endpoint = created_handle.endpoint().unwrap();
    let example_url = "https://jsonplaceholder.typicode.com/todos/1".to_owned();
    created_endpoint.url = example_url.clone();
    created_endpoint.write().unwrap();
    quartz.handle_cp(true, "testing", "testing-copy").unwrap();

    let found_handle = quartz.new_handle("testing-copy/ex2/sub");
    assert!(found_handle.exists());
    let found_endpoint = found_handle.endpoint().unwrap();
    assert_eq!(found_endpoint.url, example_url);

    // Check if the original endpoint was somehow affected
    let original_handle = quartz.new_handle("testing/ex2/sub");
    let original_endpoint = original_handle.endpoint().unwrap();
    assert_eq!(original_endpoint.url, example_url)
}

#[test]
fn test_endpoint_removal() {
    let quartz = TestQuartz::empty();
    quartz.endpoint_create("testing").unwrap();
    let handle = quartz.endpoint_create("testing/ex2").unwrap();
    handle.delete(false).unwrap();

    assert!(!quartz.new_handle("testing/ex2").exists());
    assert!(quartz.new_handle("testing").exists());
}

#[test]
fn test_recursive_endpoint_removal() {
    let quartz = TestQuartz::empty();
    let handle = quartz.endpoint_create("testing").unwrap();
    quartz.endpoint_create("testing/ex2").unwrap();
    handle.delete(true).unwrap();

    assert!(!quartz.new_handle("testing/ex2").exists());
    assert!(!quartz.new_handle("testing").exists());
}

#[test]
fn test_non_recursive_endpoint_removal() {
    let quartz = TestQuartz::empty();
    let handle = quartz.endpoint_create("testing").unwrap();
    quartz.endpoint_create("testing/ex2").unwrap();

    assert!(handle.delete(false).is_err());

    assert!(quartz.new_handle("testing/ex2").exists());
    assert!(quartz.new_handle("testing").exists());
}

#[test]
fn test_mv_handle() {
    let quartz = TestQuartz::empty();
    quartz.endpoint_create("testing").unwrap();

    quartz.handle_mv("testing", "new").unwrap();

    assert!(quartz.new_handle("new").exists());
    assert!(!quartz.new_handle("testing").exists());
}

#[test]
fn test_mv_handle_overwrite() {
    let quartz = TestQuartz::empty();
    let first_handle = quartz.endpoint_create("testing").unwrap();
    let mut first_endpoint = first_handle.endpoint().unwrap();
    first_endpoint.url = "https://jsonplaceholder.typicode.com/todos/1".to_owned();
    first_endpoint.write().unwrap();
    let second_handle = quartz.endpoint_create("new").unwrap();
    let mut second_endpoint = second_handle.endpoint().unwrap();
    second_endpoint.url = "https://jsonplaceholder.typicode.com/todos/2".to_owned();
    second_endpoint.write().unwrap();

    quartz.handle_mv("testing", "new").unwrap();

    let overwritten_handle = EndpointHandle::new(&quartz, "new".into());
    assert!(overwritten_handle.exists());
    assert!(!EndpointHandle::new(&quartz, "testing".into()).exists());

    let overwritten_endpoint = overwritten_handle.endpoint().unwrap();
    assert_eq!(
        overwritten_endpoint.url,
        "https://jsonplaceholder.typicode.com/todos/1"
    );
}

#[test]
fn test_resolve_endpoint_url() {
    let quartz = TestQuartz::empty();
    let default_env = quartz.env().unwrap();

    let first_handle = quartz.endpoint_create("jsonplaceholder").unwrap();
    let mut first_endpoint = first_handle.endpoint().unwrap();
    first_endpoint.url = "https://jsonplaceholder.typicode.com".into();
    first_endpoint.write().unwrap();

    let sub_handle = quartz.endpoint_create("jsonplaceholder/todos").unwrap();
    let mut sub_endpoint = sub_handle.endpoint().unwrap();
    sub_endpoint.url = "**/todos".into();
    sub_endpoint.write().unwrap();

    let resolved = sub_endpoint.resolved_url(&default_env);
    assert_eq!(resolved, "https://jsonplaceholder.typicode.com/todos");
}

#[test]
fn test_multilevel_inheritance() {
    let quartz = TestQuartz::empty();
    let default_env = quartz.env().unwrap();
    let first_handle = quartz.endpoint_create("jsonplaceholder").unwrap();
    let mut first_endpoint = first_handle.endpoint().unwrap();
    first_endpoint.url = "https://jsonplaceholder.typicode.com".into();
    first_endpoint.write().unwrap();

    let second_handle = quartz.endpoint_create("jsonplaceholder/todos").unwrap();
    let mut second_endpoint = second_handle.endpoint().unwrap();
    second_endpoint.url = "**/todos".into();
    second_endpoint.write().unwrap();

    let third_handle = quartz
        .endpoint_create("jsonplaceholder/todos/first")
        .unwrap();
    let mut third_endpoint = third_handle.endpoint().unwrap();
    third_endpoint.url = "**/1".into();
    third_endpoint.write().unwrap();

    let resolved = third_endpoint.resolved_url(&default_env);
    assert_eq!(resolved, "https://jsonplaceholder.typicode.com/todos/1");
}

#[test]
fn test_resolve_endpoint_vars() {
    let quartz = TestQuartz::empty();
    let mut default_env = quartz.env().unwrap();

    let first_handle = quartz.endpoint_create("jsonplaceholder").unwrap();
    let mut first_endpoint = first_handle.endpoint().unwrap();
    first_endpoint.url = "https://jsonplaceholder.typicode.com/todos/{{id}}".into();
    default_env.var_set("id".into(), "1".into()).unwrap();
    default_env.save().unwrap();
    first_endpoint.write().unwrap();

    let resolved = first_endpoint.as_resolved(&default_env);
    assert_eq!(resolved.url, "https://jsonplaceholder.typicode.com/todos/1");
}

#[test]
fn test_handle_parent() {
    let quartz = TestQuartz::empty();

    let first_handle = quartz.new_handle("first");
    let second_handle = quartz.new_handle("first/sub");
    assert_eq!(first_handle.head(), "first");
    assert_eq!(second_handle.head(), "sub");

    let second_parent = second_handle.parent().unwrap();
    assert_eq!(second_parent.head(), "first");
}
