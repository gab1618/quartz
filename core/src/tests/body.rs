use crate::{endpoint::value::Endpoint, tests::utils::TestQuartz};

#[test]
fn change_body() {
    let quartz = TestQuartz::empty();
    let endpoint = quartz.endpoint();
    let handle = endpoint.new_handle("testing");
    handle.write_endpoint(Endpoint::new()).unwrap();
    endpoint.switch("testing".to_owned()).unwrap();

    handle.set_body("{}").unwrap();
    assert_eq!(handle.body().unwrap(), "{}".to_owned());
}
