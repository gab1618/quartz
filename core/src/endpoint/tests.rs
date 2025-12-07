use crate::{endpoint::EndpointHandle, tests::utils::TestQuartz};

#[test]
fn test_simple_cp_handle() {
    let quartz = TestQuartz::empty();
    quartz.handle_create("testing").unwrap();
    quartz.handle_cp(false, "testing", "testing-copy").unwrap();

    let found_handle = EndpointHandle::new(&quartz, "testing-copy".into());
    assert!(found_handle.exists());
}

#[test]
fn test_recursive_cp_handle() {
    let quartz = TestQuartz::empty();
    quartz.handle_create("testing").unwrap();
    quartz.handle_create("testing/ex").unwrap();
    quartz.handle_create("testing/ex2").unwrap();
    quartz.handle_create("testing/ex2/sub").unwrap();
    quartz.handle_cp(true, "testing", "testing-copy").unwrap();

    assert!(EndpointHandle::new(&quartz, "testing-copy".into()).exists());
    assert!(EndpointHandle::new(&quartz, "testing-copy/ex".into()).exists());
    assert!(EndpointHandle::new(&quartz, "testing-copy/ex2".into()).exists());
    assert!(EndpointHandle::new(&quartz, "testing-copy/ex2/sub".into()).exists());
}

#[test]
fn test_non_recursive_cp_handle() {
    let quartz = TestQuartz::empty();
    quartz.handle_create("testing").unwrap();
    quartz.handle_create("testing/ex").unwrap();
    quartz.handle_create("testing/ex2").unwrap();
    quartz.handle_create("testing/ex2/sub").unwrap();
    quartz.handle_cp(false, "testing", "testing-copy").unwrap();

    assert!(EndpointHandle::new(&quartz, "testing-copy".into()).exists());
    assert!(!EndpointHandle::new(&quartz, "testing-copy/ex".into()).exists());
    assert!(!EndpointHandle::new(&quartz, "testing-copy/ex2".into()).exists());
    assert!(!EndpointHandle::new(&quartz, "testing-copy/ex2/sub".into()).exists());
}

#[test]
fn test_cp_endpoint_spec() {
    let quartz = TestQuartz::empty();
    quartz.handle_create("testing").unwrap();
    quartz.handle_create("testing/ex2").unwrap();
    let mut created_endpoint = quartz.handle_create("testing/ex2/sub").unwrap();
    let example_url = "https://jsonplaceholder.typicode.com/todos/1".to_owned();
    created_endpoint.url = example_url.clone();
    created_endpoint.write().unwrap();
    quartz.handle_cp(true, "testing", "testing-copy").unwrap();

    let found_handle = EndpointHandle::new(&quartz, "testing-copy/ex2/sub".into());
    assert!(found_handle.exists());
    let found_endpoint = found_handle.endpoint().unwrap();
    assert_eq!(found_endpoint.url, example_url);

    // Check if the original endpoint was somehow affected
    let original_handle = EndpointHandle::new(&quartz, "testing/ex2/sub".into());
    let original_endpoint = original_handle.endpoint().unwrap();
    assert_eq!(original_endpoint.url, example_url)
}

#[test]
fn test_endpoint_removal() {
    let quartz = TestQuartz::empty();
    quartz.handle_create("testing").unwrap();
    quartz.handle_create("testing/ex2").unwrap();
    quartz.handle_rm(false, "testing/ex2").unwrap();
    assert!(!EndpointHandle::new(&quartz, "testing/ex2".into()).exists());
    assert!(EndpointHandle::new(&quartz, "testing".into()).exists());
}

#[test]
fn test_recursive_endpoint_removal() {
    let quartz = TestQuartz::empty();
    quartz.handle_create("testing").unwrap();
    quartz.handle_create("testing/ex2").unwrap();
    quartz.handle_rm(true, "testing").unwrap();
    assert!(!EndpointHandle::new(&quartz, "testing/ex2".into()).exists());
    assert!(!EndpointHandle::new(&quartz, "testing".into()).exists());
}

#[test]
fn test_non_recursive_endpoint_removal() {
    let quartz = TestQuartz::empty();
    quartz.handle_create("testing").unwrap();
    quartz.handle_create("testing/ex2").unwrap();

    assert!(quartz.handle_rm(false, "testing").is_err());

    assert!(EndpointHandle::new(&quartz, "testing/ex2".into()).exists());
    assert!(EndpointHandle::new(&quartz, "testing".into()).exists());
}

#[test]
fn test_mv_handle() {
    let quartz = TestQuartz::empty();
    quartz.handle_create("testing").unwrap();

    quartz.handle_mv("testing", "new").unwrap();
    assert!(EndpointHandle::new(&quartz, "new".into()).exists());
    assert!(!EndpointHandle::new(&quartz, "testing".into()).exists());
}

#[test]
fn test_mv_handle_overwrite() {
    let quartz = TestQuartz::empty();
    let mut first_endpoint = quartz.handle_create("testing").unwrap();
    first_endpoint.url = "https://jsonplaceholder.typicode.com/todos/1".to_owned();
    first_endpoint.write().unwrap();
    let mut second_endpoint = quartz.handle_create("new").unwrap();
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

    let mut first_endpoint = quartz.handle_create("jsonplaceholder").unwrap();
    first_endpoint.url = "https://jsonplaceholder.typicode.com".into();
    first_endpoint.write().unwrap();

    let mut sub_endpoint = quartz.handle_create("jsonplaceholder/todos").unwrap();
    sub_endpoint.url = "**/todos".into();
    sub_endpoint.write().unwrap();

    let resolved = sub_endpoint.resolved_url();
    assert_eq!(resolved, "https://jsonplaceholder.typicode.com/todos");
}

#[test]
fn test_multilevel_inheritance() {
    let quartz = TestQuartz::empty();
    let mut first_endpoint = quartz.handle_create("jsonplaceholder").unwrap();
    first_endpoint.url = "https://jsonplaceholder.typicode.com".into();
    first_endpoint.write().unwrap();

    let mut second_endpoint = quartz.handle_create("jsonplaceholder/todos").unwrap();
    second_endpoint.url = "**/todos".into();
    second_endpoint.write().unwrap();

    let mut third_endpoint = quartz.handle_create("jsonplaceholder/todos/first").unwrap();
    third_endpoint.url = "**/1".into();
    third_endpoint.write().unwrap();

    let resolved = third_endpoint.resolved_url();
    assert_eq!(resolved, "https://jsonplaceholder.typicode.com/todos/1");
}

#[test]
fn test_resolve_endpoint_vars() {
    let quartz = TestQuartz::empty();

    let mut first_endpoint = quartz.handle_create("jsonplaceholder").unwrap();
    first_endpoint.url = "https://jsonplaceholder.typicode.com/todos/{{id}}".into();
    first_endpoint.variables.insert("id".into(), "1".into());
    first_endpoint.write().unwrap();

    let resolved = first_endpoint.as_resolved();
    assert_eq!(resolved.url, "https://jsonplaceholder.typicode.com/todos/1");
}
