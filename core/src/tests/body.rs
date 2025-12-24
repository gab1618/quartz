use crate::{endpoint::Endpoint, tests::utils::TestQuartz};

#[test]
fn change_body() {
    let quartz = TestQuartz::empty();
    let handle = quartz.new_handle("testing");
    let endpoint = Endpoint::new();
    handle.write_endpoint(endpoint).unwrap();
    quartz.handle_switch("testing".to_owned()).unwrap();
    quartz.set_body("{}".to_owned()).unwrap();
    assert_eq!(quartz.get_body().unwrap(), "{}".to_owned());
}
