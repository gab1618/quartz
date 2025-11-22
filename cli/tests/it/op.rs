use crate::utils::{Quartz, TestResult};

#[test]
pub fn mv() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    let output = quartz.cmd(&["mv", "httpbin/get", "httpbin/getter"])?;
    assert!(output.status.success(), "{}", output.stderr);

    let _output = quartz.cmd(&["use", "httpbin/get"])?;

    let output = quartz.cmd(&["-x", "httpbin/getter", "show", "url"])?;
    assert_eq!(output.stdout.trim(), "{{BASE_URL}}/get");

    Ok(())
}

#[test]
pub fn mv_overwrite_empty() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    let output = quartz.cmd(&["mv", "httpbin/get", "httpbin"])?;
    assert!(output.status.success(), "{}", output.stderr);

    let _output = quartz.cmd(&["use", "httpbin/get"])?;

    let output = quartz.cmd(&["-x", "httpbin", "show", "url"])?;
    assert_eq!(output.stdout.trim(), "{{BASE_URL}}/get");

    Ok(())
}

/// When moving a single endpoint to an existent endpoint, overwrite it.
#[test]
pub fn mv_overwrite() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    let output = quartz.cmd(&["mv", "httpbin/get", "httpbin"])?;
    assert!(output.status.success(), "{}", output.stderr);

    let _output = quartz.cmd(&["use", "httpbin/get"])?;

    let output = quartz.cmd(&["-x", "httpbin", "show", "url"])?;
    assert_eq!(output.stdout.trim(), "{{BASE_URL}}/get");

    Ok(())
}

/// When selecting multiple handles to mv at once, it should nest them into the last argument.
///
/// ```bash
/// quartz mv e1 e2 group
/// ```
///
/// No matter if `group` exists or not, it should result in:
/// * group
///     * e1
///     * e2
#[test]
pub fn mv_multiple() -> TestResult {
    let quartz = Quartz::preset_httpbin()?;

    let output = quartz.cmd(&[
        "mv",
        "httpbin/redirect/absolute",
        "httpbin/redirect/relative",
        "httpbin",
    ])?;
    assert!(output.status.success(), "{}", output.stderr);

    let stdout = quartz.cmd(&["ls"])?.stdout;
    assert!(stdout.contains("httpbin/absolute"), "{}", stdout);
    assert!(stdout.contains("httpbin/relative"), "{}", stdout);

    Ok(())
}
