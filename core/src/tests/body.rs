use crate::tests::utils::TestQuartz;

#[test]
fn change_body() {
    let quartz = TestQuartz::empty();
    quartz.handle_create("testing").unwrap();
    quartz.handle_switch("testing".to_owned()).unwrap();
    quartz.set_body("{}".to_owned()).unwrap();
    assert_eq!(quartz.get_body().unwrap(), "{}".to_owned());
}
