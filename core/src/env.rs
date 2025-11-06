use std::{
    collections::HashMap,
    fmt::Display,
    io::Write,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::{cookie::CookieJar, endpoint::Headers, pairmap::PairMap, QuartzError, QuartzResult};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Variables(pub HashMap<String, String>);

impl Deref for Variables {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Variables {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Variables {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, value) in self.iter() {
            writeln!(f, "{key}={value}")?;
        }

        Ok(())
    }
}

impl PairMap<'_> for Variables {
    const NAME: &'static str = "variable";

    fn map(&mut self) -> &mut HashMap<String, String> {
        &mut self.0
    }
}

impl Variables {
    pub fn parse(file_content: &str) -> Self {
        let mut variables = Variables::default();

        for var in file_content.split('\n').filter(|line| !line.is_empty()) {
            variables.set(var).unwrap();
        }

        variables
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Env {
    mount_path: PathBuf,
    pub name: String,
    pub variables: Variables,
    pub headers: Headers,
}

impl Env {
    pub fn new(name: &str, mount_path: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            ..Self::default(mount_path)
        }
    }
    fn default(mount_path: PathBuf) -> Self {
        Self {
            mount_path,
            name: String::from("default"),
            variables: Variables::default(),
            headers: Headers::default(),
        }
    }

    pub fn dir(&self) -> PathBuf {
        self.mount_path.join("env").join(&self.name)
    }

    pub fn write(&self) -> QuartzResult {
        let dir = self.dir();

        std::fs::create_dir(dir).map_err(|_| QuartzError::Internal)?;

        self.update().map_err(|_| QuartzError::Internal)?;

        Ok(())
    }

    pub fn update(&self) -> QuartzResult {
        let mut var_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.dir().join("variables"))
            .map_err(|_| QuartzError::Internal)?;
        let mut headers_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.dir().join("headers"))
            .map_err(|_| QuartzError::Internal)?;

        if !self.variables.is_empty() {
            var_file
                .write_all(format!("{}", self.variables).as_bytes())
                .map_err(|_| QuartzError::Internal)?;
        }
        if !self.headers.0.is_empty() {
            headers_file
                .write_all(format!("{}", self.headers).as_bytes())
                .map_err(|_| QuartzError::Internal)?;
        }

        Ok(())
    }

    /// Returns `true` if this environment already exists on the quartz project.
    pub fn exists(&self) -> bool {
        self.dir().exists()
    }

    pub fn parse(mount_path: PathBuf, name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut env = Self::new(name, mount_path);

        if let Ok(var_contents) = std::fs::read_to_string(env.dir().join("variables")) {
            env.variables = Variables::parse(&var_contents);
        }
        if let Ok(header_contents) = std::fs::read_to_string(env.dir().join("headers")) {
            env.headers = Headers::parse(&header_contents);
        }

        Ok(env)
    }


    pub fn cookie_jar(&self) -> CookieJar {
        let path = self.dir().join(CookieJar::FILENAME);
        let mut jar = CookieJar::read(&path).unwrap_or_default();

        jar.path = path;

        jar
    }
}
