use crate::{endpoint::EndpointHandle, tests::utils::TestQuartz};

#[test]
fn test_simple_cp_handle() {
    let quartz = TestQuartz::empty().unwrap();
    quartz.handle_create("testing").unwrap();
    quartz.handle_cp(false, "testing", "testing-copy").unwrap();

    let found_handle = EndpointHandle::from("testing-copy");
    assert!(found_handle.exists(&quartz.inner));
}

#[test]
fn test_recursive_cp_handle() {
    let quartz = TestQuartz::empty().unwrap();
    quartz.handle_create("testing").unwrap();
    quartz.handle_create("testing/ex").unwrap();
    quartz.handle_create("testing/ex2").unwrap();
    quartz.handle_create("testing/ex2/sub").unwrap();
    quartz.handle_cp(true, "testing", "testing-copy").unwrap();

    assert!(EndpointHandle::from("testing-copy").exists(&quartz.inner));
    assert!(EndpointHandle::from("testing-copy/ex").exists(&quartz.inner));
    assert!(EndpointHandle::from("testing-copy/ex2").exists(&quartz.inner));
    assert!(EndpointHandle::from("testing-copy/ex2/sub").exists(&quartz.inner));
}

#[test]
fn test_non_recursive_cp_handle() {
    let quartz = TestQuartz::empty().unwrap();
    quartz.handle_create("testing").unwrap();
    quartz.handle_create("testing/ex").unwrap();
    quartz.handle_create("testing/ex2").unwrap();
    quartz.handle_create("testing/ex2/sub").unwrap();
    quartz.handle_cp(false, "testing", "testing-copy").unwrap();

    assert!(EndpointHandle::from("testing-copy").exists(&quartz.inner));
    assert!(!EndpointHandle::from("testing-copy/ex").exists(&quartz.inner));
    assert!(!EndpointHandle::from("testing-copy/ex2").exists(&quartz.inner));
    assert!(!EndpointHandle::from("testing-copy/ex2/sub").exists(&quartz.inner));
}

#[test]
fn test_cp_endpoint_spec() {
    let quartz = TestQuartz::empty().unwrap();
    quartz.handle_create("testing").unwrap();
    quartz.handle_create("testing/ex2").unwrap();
    let mut created_endpoint = quartz.handle_create("testing/ex2/sub").unwrap();
    let example_url = "https://jsonplaceholder.typicode.com/todos/1".to_owned();
    created_endpoint.url = example_url.clone();
    created_endpoint.write().unwrap();
    quartz.handle_cp(true, "testing", "testing-copy").unwrap();

    let found_handle = EndpointHandle::from("testing-copy/ex2/sub");
    assert!(found_handle.exists(&quartz.inner));
    let found_endpoint = found_handle.endpoint(&quartz.inner).unwrap();
    assert_eq!(found_endpoint.url, example_url);

    // Check if the original endpoint was somehow affected
    let original_handle = EndpointHandle::from("testing/ex2/sub");
    let original_endpoint = original_handle.endpoint(&quartz.inner).unwrap();
    assert_eq!(original_endpoint.url, example_url)
}

#[test]
fn test_endpoint_removal() {
    let quartz = TestQuartz::empty().unwrap();
    quartz.handle_create("testing").unwrap();
    quartz.handle_create("testing/ex2").unwrap();
    quartz.handle_rm(false, "testing/ex2").unwrap();
    assert!(!EndpointHandle::from("testing/ex2").exists(&quartz.inner));
    assert!(EndpointHandle::from("testing").exists(&quartz.inner));
}

#[test]
fn test_recursive_endpoint_removal() {
    let quartz = TestQuartz::empty().unwrap();
    quartz.handle_create("testing").unwrap();
    quartz.handle_create("testing/ex2").unwrap();
    quartz.handle_rm(true, "testing").unwrap();
    assert!(!EndpointHandle::from("testing/ex2").exists(&quartz.inner));
    assert!(!EndpointHandle::from("testing").exists(&quartz.inner));
}

#[test]
fn test_non_recursive_endpoint_removal() {
    let quartz = TestQuartz::empty().unwrap();
    quartz.handle_create("testing").unwrap();
    quartz.handle_create("testing/ex2").unwrap();

    assert!(quartz.handle_rm(false, "testing").is_err());

    assert!(EndpointHandle::from("testing/ex2").exists(&quartz.inner));
    assert!(EndpointHandle::from("testing").exists(&quartz.inner));
}

#[test]
fn test_mv_handle() {
    let quartz = TestQuartz::empty().unwrap();
    quartz.handle_create("testing").unwrap();

    quartz.handle_mv("testing", "new").unwrap();
    assert!(EndpointHandle::from("new").exists(&quartz.inner));
    assert!(!EndpointHandle::from("testing").exists(&quartz.inner));
}

#[test]
fn test_mv_handle_overwrite() {
    let quartz = TestQuartz::empty().unwrap();
    let mut first_endpoint = quartz.handle_create("testing").unwrap();
    first_endpoint.url = "https://jsonplaceholder.typicode.com/todos/1".to_owned();
    first_endpoint.write().unwrap();
    let mut second_endpoint = quartz.handle_create("new").unwrap();
    second_endpoint.url = "https://jsonplaceholder.typicode.com/todos/2".to_owned();
    second_endpoint.write().unwrap();

    quartz.handle_mv("testing", "new").unwrap();

    let overwritten_handle = EndpointHandle::from("new");
    assert!(overwritten_handle.exists(&quartz.inner));
    assert!(!EndpointHandle::from("testing").exists(&quartz.inner));

    let overwritten_endpoint = overwritten_handle.endpoint(&quartz.inner).unwrap();
    assert_eq!(
        overwritten_endpoint.url,
        "https://jsonplaceholder.typicode.com/todos/1"
    );
}
