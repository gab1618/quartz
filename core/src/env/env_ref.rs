use std::{io::Write, path::PathBuf};

use crate::{
    Quartz, Result,
    cookie::CookieJar,
    env::{
        env::{Env, Variables},
        error::EnvError,
    },
    headers::Headers,
};
#[derive(Clone)]
pub struct EnvRef<'a> {
    pub name: String,
    quartz: &'a Quartz,
}

impl<'a> EnvRef<'a> {
    pub fn new(quartz: &'a Quartz, name: String) -> Self {
        Self { quartz, name }
    }
    pub fn read(&self) -> Result<Env> {
        let mut env = Env::default();

        if let Ok(var_contents) = std::fs::read_to_string(self.dir().join("variables")) {
            env.variables = Variables::parse(&var_contents)?;
        }
        if let Ok(header_contents) = std::fs::read_to_string(self.dir().join("headers")) {
            env.headers = Headers::parse(&header_contents)?;
        }

        Ok(env)
    }
    pub fn exists(&self) -> bool {
        self.dir().exists()
    }

    pub fn dir(&self) -> PathBuf {
        self.quartz.path.join("env").join(&self.name)
    }

    pub fn save(&self, env: &Env) -> Result {
        let dir = self.dir();
        if !dir.exists() {
            std::fs::create_dir(&dir).map_err(EnvError::CreateEnvDir)?;
        }
        let mut var_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(dir.join("variables"))
            .map_err(EnvError::UpdateVariablesFile)?;
        let mut headers_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(dir.join("headers"))
            .map_err(EnvError::UpdateHeadersFile)?;

        if !env.variables.is_empty() {
            var_file
                .write_all(format!("{}", env.variables).as_bytes())
                .map_err(EnvError::UpdateVariablesFile)?;
        }
        if !env.headers.0.is_empty() {
            headers_file
                .write_all(format!("{}", env.headers).as_bytes())
                .map_err(EnvError::UpdateHeadersFile)?;
        }

        Ok(())
    }
    pub fn clean(&self) -> Result {
        std::fs::remove_dir_all(self.dir()).map_err(EnvError::DeleteEnv)?;
        Ok(())
    }

    pub fn cookie_jar(&self) -> CookieJar {
        let path = self.dir().join(CookieJar::FILENAME);
        let mut jar = CookieJar::read(&path).unwrap_or_default();

        jar.path = path;

        jar
    }
}
