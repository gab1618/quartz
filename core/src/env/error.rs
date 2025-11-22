#[derive(Debug, thiserror::Error)]
pub enum EnvError {
    #[error("Env already exists")]
    AlreadyExistingEnv,
    #[error("Could not get envs: {0}")]
    GetEnvs(#[source] std::io::Error),
    #[error("Could not parse env filename")]
    ParseEnvName,
    #[error("Could not proceed. Env {0} is in use")]
    EnvInUse(String),
    #[error("Could not delete env: {0}")]
    DeleteEnv(#[source] std::io::Error),
    #[error("Env not found")]
    NotFound,
    #[error("Could not create env dir: {0}")]
    CreateEnvDir(#[source] std::io::Error),
    #[error("Could not update variables file: {0}")]
    UpdateVariablesFile(#[source] std::io::Error),
    #[error("Could not update headers file: {0}")]
    UpdateHeadersFile(#[source] std::io::Error),
}
