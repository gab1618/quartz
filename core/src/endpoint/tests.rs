use super::{endpoint::Endpoint, handle::EndpointHandle};
use crate::tests::utils::TestQuartz;

#[test]
fn test_simple_cp_handle() {
    let quartz = TestQuartz::empty();
    let endpoint = quartz.endpoint();
    let handle = endpoint.new_handle("testing");
    handle.ensure_dir().unwrap();
    endpoint
        .copy(false, "testing", "testing-copy")
        .unwrap();

    let found_handle = endpoint.new_handle("testing-copy");
    assert!(found_handle.exists());
}

#[test]
fn test_recursive_cp_handle() {
    let quartz = TestQuartz::empty();
    let endpoint = quartz.endpoint();
    endpoint.new_handle("testing").ensure_dir().unwrap();
    endpoint.new_handle("testing/ex").ensure_dir().unwrap();
    endpoint.new_handle("testing/ex2").ensure_dir().unwrap();
    endpoint.new_handle("testing/ex2/sub").ensure_dir().unwrap();
    endpoint.copy(true, "testing", "testing-copy").unwrap();

    assert!(endpoint.new_handle("testing-copy").exists());
    assert!(endpoint.new_handle("testing-copy/ex").exists());
    assert!(endpoint.new_handle("testing-copy/ex2").exists());
    assert!(endpoint.new_handle("testing-copy/ex2/sub").exists());
}

#[test]
fn test_non_recursive_cp_handle() {
    let quartz = TestQuartz::empty();
    let endpoint = quartz.endpoint();
    endpoint.new_handle("testing").ensure_dir().unwrap();
    endpoint.new_handle("testing/ex").ensure_dir().unwrap();
    endpoint.new_handle("testing/ex2").ensure_dir().unwrap();
    endpoint.new_handle("testing/ex2/sub").ensure_dir().unwrap();
    endpoint
        .copy(false, "testing", "testing-copy")
        .unwrap();

    assert!(endpoint.new_handle("testing-copy").exists());
    assert!(!endpoint.new_handle("testing-copy/ex").exists());
    assert!(!endpoint.new_handle("testing-copy/ex2").exists());
    assert!(!endpoint.new_handle("testing-copy/ex2/sub").exists());
}

#[test]
fn test_cp_endpoint_spec() {
    let quartz = TestQuartz::empty();
    let endpoint = quartz.endpoint();
    endpoint.new_handle("testing").ensure_dir().unwrap();
    endpoint.new_handle("testing/ex2").ensure_dir().unwrap();
    let created_handle = endpoint.new_handle("testing/ex2/sub");

    let example_url = "https://jsonplaceholder.typicode.com/todos/1".to_owned();
    let mut new_endpoint = Endpoint::new();
    new_endpoint.url = example_url.clone();
    created_handle.write_endpoint(new_endpoint).unwrap();

    let mut created_endpoint = created_handle.endpoint().unwrap();
    created_endpoint.url = example_url.clone();
    created_handle.write_endpoint(created_endpoint).unwrap();
    endpoint.copy(true, "testing", "testing-copy").unwrap();

    let found_handle = endpoint.new_handle("testing-copy/ex2/sub");
    assert!(found_handle.exists());
    let found_endpoint = found_handle.endpoint().unwrap();
    assert_eq!(found_endpoint.url, example_url);

    // Check if the original endpoint was somehow affected
    let original_handle = endpoint.new_handle("testing/ex2/sub");
    let original_endpoint = original_handle.endpoint().unwrap();
    assert_eq!(original_endpoint.url, example_url)
}

#[test]
fn test_endpoint_removal() {
    let quartz = TestQuartz::empty();
    let endpoint = quartz.endpoint();
    endpoint.new_handle("testing").ensure_dir().unwrap();
    let handle = endpoint.new_handle("testing/ex2");
    handle.ensure_dir().unwrap();
    handle.delete(false).unwrap();

    assert!(!endpoint.new_handle("testing/ex2").exists());
    assert!(endpoint.new_handle("testing").exists());
}

#[test]
fn test_recursive_endpoint_removal() {
    let quartz = TestQuartz::empty();
    let endpoint = quartz.endpoint();
    let handle = endpoint.new_handle("testing");
    handle.ensure_dir().unwrap();
    endpoint.new_handle("testing/ex2").ensure_dir().unwrap();
    handle.delete(true).unwrap();

    assert!(!endpoint.new_handle("testing/ex2").exists());
    assert!(!endpoint.new_handle("testing").exists());
}

#[test]
fn test_non_recursive_endpoint_removal() {
    let quartz = TestQuartz::empty();
    let endpoint = quartz.endpoint();
    let handle = endpoint.new_handle("testing");
    handle.ensure_dir().unwrap();
    endpoint.new_handle("testing/ex2").ensure_dir().unwrap();

    assert!(handle.delete(false).is_err());

    assert!(endpoint.new_handle("testing/ex2").exists());
    assert!(endpoint.new_handle("testing").exists());
}

#[test]
fn test_mv_handle() {
    let quartz = TestQuartz::empty();
    let endpoint = quartz.endpoint();
    endpoint.new_handle("testing").ensure_dir().unwrap();

    endpoint.mv("testing", "new").unwrap();

    assert!(endpoint.new_handle("new").exists());
    assert!(!endpoint.new_handle("testing").exists());
}

#[test]
fn test_mv_handle_overwrite() {
    let quartz = TestQuartz::empty();
    let endpoint = quartz.endpoint();
    let first_handle = endpoint.new_handle("testing");
    let mut first_endpoint = Endpoint::new();
    first_endpoint.url = "https://jsonplaceholder.typicode.com/todos/1".to_owned();
    first_handle.write_endpoint(first_endpoint).unwrap();
    let second_handle = endpoint.new_handle("new");
    let mut second_endpoint = Endpoint::new();
    second_endpoint.url = "https://jsonplaceholder.typicode.com/todos/2".to_owned();
    second_handle.write_endpoint(second_endpoint).unwrap();

    endpoint.mv("testing", "new").unwrap();

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
    let env = quartz.env();
    let default_env = env.current().unwrap();
    let endpoint = quartz.endpoint();

    let first_handle = endpoint.new_handle("jsonplaceholder");

    let mut first_endpoint = Endpoint::new();
    first_endpoint.url = "https://jsonplaceholder.typicode.com".into();
    first_handle.write_endpoint(first_endpoint).unwrap();

    let sub_handle = endpoint.new_handle("jsonplaceholder/todos");
    let mut sub_endpoint = Endpoint::new();
    sub_endpoint.url = "**/todos".into();
    sub_handle.write_endpoint(&sub_endpoint).unwrap();

    let resolved = sub_endpoint
        .resolved_url(&sub_handle, &default_env)
        .unwrap();
    assert_eq!(resolved, "https://jsonplaceholder.typicode.com/todos");
}

#[test]
fn test_multilevel_inheritance() {
    let quartz = TestQuartz::empty();
    let env = quartz.env();
    let default_env = env.current().unwrap();
    let endpoint = quartz.endpoint();
    let first_handle = endpoint.new_handle("jsonplaceholder");
    let mut first_endpoint = Endpoint::new();
    first_endpoint.url = "https://jsonplaceholder.typicode.com".into();
    first_handle.write_endpoint(first_endpoint).unwrap();

    let second_handle = endpoint.new_handle("jsonplaceholder/todos");
    let mut second_endpoint = Endpoint::new();
    second_endpoint.url = "**/todos".into();
    second_handle.write_endpoint(second_endpoint).unwrap();

    let third_handle = endpoint.new_handle("jsonplaceholder/todos/first");
    let mut third_endpoint = Endpoint::new();
    third_endpoint.url = "**/1".into();
    third_handle.write_endpoint(&third_endpoint).unwrap();

    let resolved = third_endpoint
        .resolved_url(&third_handle, &default_env)
        .unwrap();
    assert_eq!(resolved, "https://jsonplaceholder.typicode.com/todos/1");
}

#[test]
fn test_resolve_endpoint_vars() {
    let quartz = TestQuartz::empty();
    let env = quartz.env();
    let mut default_env = env.current().unwrap();
    let endpoint = quartz.endpoint();

    let first_handle = endpoint.new_handle("jsonplaceholder");
    let mut first_endpoint = Endpoint::new();
    first_endpoint.url = "https://jsonplaceholder.typicode.com/todos/{{id}}".into();
    first_handle.write_endpoint(&first_endpoint).unwrap();

    default_env.var_set("id".into(), "1".into()).unwrap();
    default_env.save().unwrap();

    let resolved = first_endpoint
        .as_resolved(&first_handle, &default_env)
        .unwrap();
    assert_eq!(resolved.url, "https://jsonplaceholder.typicode.com/todos/1");
}

#[test]
fn test_handle_parent() {
    let quartz = TestQuartz::empty();
    let endpoint = quartz.endpoint();

    let first_handle = endpoint.new_handle("first");
    let second_handle = endpoint.new_handle("first/sub");
    assert_eq!(first_handle.head(), "first");
    assert_eq!(second_handle.head(), "sub");

    let second_parent = second_handle.parent().unwrap();
    assert_eq!(second_parent.head(), "first");
}

#[test]
fn test_resolve_body() {
    let quartz = TestQuartz::empty();
    let env = quartz.env();
    let mut default_env = env.current().unwrap();
    let endpoint = quartz.endpoint();

    let first_handle = endpoint.new_handle("first");

    assert_eq!(first_handle.body(), None);
    first_handle.set_body("{{testing}}".into()).unwrap();

    let raw_body = first_handle.body();
    assert_eq!(raw_body, Some("{{testing}}".into()));

    default_env.var_set("testing".into(), "1".into()).unwrap();
    assert_eq!(first_handle.resolved_body(&default_env), Some("1".into()));
}
