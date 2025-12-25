use crate::{Error, Result};

/// Validator for files that don't have to do any checks. It is
/// garanteed to return [`Ok`].
///
/// # Examples
///
/// ```
/// use quartz_core::validator;
///
/// // Totally broken JSON input
/// let input = r#"
/// {
///     "value":
/// "#;
///
/// assert!(validator::infallible(input).is_ok());
/// ```
pub fn infallible(_input: &str) -> Result {
    Ok(())
}

/// Checks if a string is valid TOML for `T`.
///
/// # Examples
///
/// ```
/// use serde::Deserialize;
/// use quartz_core::validator;
///
/// #[derive(Deserialize)]
/// struct Config {
///     title: String,
///     owner: Owner,
/// }
///
/// #[derive(Deserialize)]
/// struct Owner {
///     name: String,
/// }
///
/// let input = r#"
///     title = 'TOML Example'
///
///     [owner]
///     name = 'Lisa'
/// "#;
///
/// let input_missing = r#"
///     title = 'TOML Example'
///
///     [owner]
/// "#;
///
///
/// assert!(validator::toml_as::<Config>(input).is_ok());
/// assert!(validator::toml_as::<Config>(input_missing).is_err());
/// ```
pub fn toml_as<T>(input: &str) -> Result
where
    T: serde::de::DeserializeOwned,
{
    toml::from_str::<T>(input).map_err(|_| Error::Internal)?;

    Ok(())
}
