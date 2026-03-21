use std::{
    fs::OpenOptions,
    io::Write,
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use crate::{
    Quartz,
    endpoint::{
        value::{Endpoint, EndpointPatch},
        error::EndpointError,
    },
    env::value::Env,
    error::Result,
};

#[derive(Clone)]
pub struct EndpointHandlePath(pub Vec<String>);

impl Deref for EndpointHandlePath {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for EndpointHandlePath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for EndpointHandlePath
where
    T: AsRef<str>,
{
    fn from(value: T) -> Self {
        let path: Vec<String> = value
            .as_ref()
            .trim_matches('/')
            .split('/')
            .map(|s| s.to_string())
            .collect();

        Self(path)
    }
}

#[derive(Clone)]
pub struct EndpointHandle<'a> {
    quartz: &'a Quartz,
    /// List of ordered parent names
    pub path: EndpointHandlePath,
}
impl<'a> EndpointHandle<'a> {
    pub fn new(quartz: &'a Quartz, path: EndpointHandlePath) -> Self {
        Self { quartz, path }
    }
    pub fn root(quartz: &'a Quartz) -> Self {
        Self::new(quartz, EndpointHandlePath(vec![]))
    }

    pub fn head(&self) -> String {
        self.path.last().unwrap_or(&String::new()).clone()
    }

    pub fn dir(&self) -> PathBuf {
        self.quartz
            .path()
            .join("endpoints")
            .join(self.path.join("/"))
    }
    pub fn endpoint_file_path(&self) -> PathBuf {
        self.dir().join("endpoint.toml")
    }
    pub fn parent(&self) -> Result<Self> {
        let mut parent_path = self.path.clone();
        if parent_path.pop().is_none() {
            return Err(EndpointError::NoHandleParentDir.into());
        }
        Ok(Self::new(self.quartz, parent_path))
    }

    pub fn handle(&self) -> String {
        self.path.join("/")
    }

    pub fn exists(&self) -> bool {
        let path = self.dir();
        path.exists()
    }
    pub fn ensure_dir(&self) -> Result {
        std::fs::create_dir_all(self.dir()).map_err(EndpointError::SaveEndpoint)?;

        Ok(())
    }

    pub fn write_endpoint<E: AsRef<Endpoint>>(&self, endpoint: &E) -> Result {
        let endpoint = endpoint.as_ref();
        let toml_content = endpoint.to_toml()?;
        self.ensure_dir()?;

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.dir().join("endpoint.toml"))
            .map_err(EndpointError::SaveEndpoint)?;

        file.write_all(toml_content.as_bytes())
            .map_err(EndpointError::SaveEndpoint)?;
        Ok(())
    }

    /// Removes endpoint to make it an empty handle
    pub fn make_empty(&self) {
        if self.endpoint().is_ok() {
            let _ = std::fs::remove_file(self.dir().join("endpoint.toml"));
            let _ = std::fs::remove_file(self.dir().join("body"));
        }
    }

    pub fn depth(&self) -> usize {
        self.path.len()
    }

    pub fn children(&self) -> Result<Vec<EndpointHandle<'_>>> {
        // If can't read the dir, just assume no children
        let paths = std::fs::read_dir(self.dir())
            .into_iter()
            .flatten()
            .flatten();
        let valid_paths = paths.map(|entry| entry.path()).filter(|path| path.is_dir());

        // TODO: add proper error handling
        let list = valid_paths
            .map(|path| {
                let quartz_endpoints_path = self.quartz.path().join("endpoints");

                let handle_subpath = path.strip_prefix(quartz_endpoints_path).unwrap();
                let handle_name = handle_subpath.as_os_str().to_str().unwrap();

                Ok(EndpointHandle::new(self.quartz, handle_name.into()))
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(list)
    }

    pub fn endpoint(&self) -> Result<Endpoint> {
        Endpoint::from_dir(&self.dir())
    }

    pub fn replace(&mut self, from: &str, to: &str) {
        let handle = self.handle().replace(from, to);
        self.path = EndpointHandle::new(self.quartz, handle.into()).path;
    }
    pub fn delete(&self, recursive: bool) -> Result {
        if !self.exists() {
            return Err(EndpointError::HandleNotFound(self.handle()).into());
        }

        if !self.children()?.is_empty() && !recursive {
            return Err(EndpointError::RemoveChildrenOnNonRecursiveMode.into());
        }

        std::fs::remove_dir_all(self.dir()).map_err(EndpointError::RemoveHandleFiles)?;

        Ok(())
    }
    pub fn body_file_path(&self) -> PathBuf {
        self.dir().join("body")
    }
    pub fn body(&self) -> Option<String> {
        std::fs::read_to_string(self.body_file_path()).ok()
    }

    fn key_match_str(key: &str) -> String {
        format!("{{{{{}}}}}", key)
    }
    pub fn resolved_body(&self, env: &Env) -> Option<String> {
        let raw_body = self.body();
        raw_body.map(|mut inner| {
            for (key, value) in env.vars().iter() {
                let key_match = Self::key_match_str(key);
                inner = inner.replace(&key_match, value);
            }
            inner
        })
    }
    pub fn set_body(&self, body: &str) -> Result {
        self.ensure_dir()?;
        let mut f = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.body_file_path())
            .map_err(EndpointError::ModifyBody)?;
        f.write_all(body.as_bytes())
            .map_err(EndpointError::ModifyBody)?;

        Ok(())
    }
    pub fn apply_endpoint_patch(&self, mut patch: EndpointPatch) -> Result {
        let mut endpoint = self.endpoint().ok().unwrap_or_default();
        endpoint.update(&mut patch)?;
        self.write_endpoint(&endpoint)?;

        Ok(())
    }
}
