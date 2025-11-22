#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Could not serialize config file: {0}")]
    SerializeConfig(#[source] toml::ser::Error),
    #[error("Invalid config key: {0}")]
    InvalidConfigKey(String),
    #[error("Could not save config: {0}")]
    SaveConfig(#[source] std::io::Error),
}

pub type ConfigResult<T = ()> = Result<T, ConfigError>;
