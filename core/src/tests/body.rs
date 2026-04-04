use crate::{endpoint::value::Endpoint, tests::utils::TestQuartz};

#[test]
fn change_body() {
    let quartz = TestQuartz::empty();
    let handle = quartz.new_handle("testing");
    handle.write_endpoint(&Endpoint::new()).unwrap();
    quartz.switch("testing".to_owned()).unwrap();

    handle.set_body("{}").unwrap();
    assert_eq!(handle.body().unwrap(), "{}".to_owned());
}
