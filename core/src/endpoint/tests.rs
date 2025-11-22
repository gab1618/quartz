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
