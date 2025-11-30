use crate::utils::*;

#[test]
fn it_adds_new_header() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    let set_output = quartz.cmd(&["header", "set", "Content-type", "application/json"])?;
    let output = quartz.cmd(&["header", "get", "Content-type"])?;

    assert!(set_output.status.success(), "{}", set_output.stderr);
    assert_eq!(output.stdout.trim(), "application/json");

    Ok(())
}

#[test]
fn it_overwrites_existing_headers() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    quartz.cmd(&["header", "set", "Content-type", "application/json"])?;
    quartz.cmd(&["header", "set", "Accept", "application/json"])?;

    let edit_output = quartz.cmd(&["header", "set", "Content-type", "plain/text"])?;
    let output = quartz.cmd(&["header", "ls"])?;

    assert!(edit_output.status.success(), "{}", edit_output.stdout);

    assert!(
        !output.stdout.contains("Content-type: application/json"),
        "old value found"
    );
    assert!(
        output.stdout.contains("Content-type: plain/text"),
        "new value was not saved"
    );

    Ok(())
}

#[test]
fn it_removes_header_by_key() -> TestResult {
    let quartz = Quartz::preset_using_sample_endpoint()?;

    quartz.cmd(&["header", "set", "Content-type", "application/json"])?;

    quartz.cmd(&["header", "set", "Accept", "form"])?;

    let remove_output = quartz.cmd(&["header", "rm", "Content-type"])?;
    assert!(remove_output.status.success(), "{}", remove_output.stderr);

    let list_output = quartz.cmd(&["header", "ls"])?;
    assert!(
        !list_output.stdout.contains("Content-type"),
        "did not remove specified header"
    );
    assert!(
        list_output.stdout.contains("Accept"),
        "removed specified header, but unrelated header is missing"
    );

    Ok(())
}
