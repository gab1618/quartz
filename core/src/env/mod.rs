use crate::{
    Quartz,
    env::{env_ref::EnvRef, error::EnvError},
    error::Result,
    state::field::StateField,
};

pub mod env;
pub mod env_ref;
pub mod error;

pub struct EnvManager<'a> {
    quartz: &'a Quartz,
}

impl<'a> EnvManager<'a> {
    pub fn new(quartz: &'a Quartz) -> Self {
        Self { quartz }
    }
    pub fn env(&self) -> Result<EnvRef<'_>> {
        let curr_env_name = self
            .quartz
            .state()
            .get(StateField::Env)
            .unwrap_or("default".into());

        let parsed_env = EnvRef::new(self.quartz, curr_env_name)?;
        Ok(parsed_env)
    }
    pub fn get_env(&self, name: String) -> Option<EnvRef<'_>> {
        let env = EnvRef::new(self.quartz, name).ok();
        env
    }
    pub fn create_env(&self, name: String) -> Result<EnvRef<'_>> {
        let new_env = EnvRef::new(self.quartz, name)?;

        new_env.save()?;

        Ok(new_env)
    }
    pub fn get_envs(&self) -> Result<impl Iterator<Item = Result<String>>> {
        let entries =
            std::fs::read_dir(self.quartz.path().join("env")).map_err(EnvError::GetEnvs)?;
        let env_names = entries.map(|entry| {
            let ok_dir_entry = entry.map_err(EnvError::GetEnvs)?;
            let filename = ok_dir_entry.file_name();
            let str_filename = filename.to_str().ok_or(EnvError::ParseEnvName)?.to_owned();
            Ok(str_filename)
        });

        Ok(env_names)
    }
    pub fn switch_env(&self, name: String) -> Result<EnvRef<'_>> {
        let requested_env = EnvRef::new(self.quartz, name)?;
        if !requested_env.exists() {
            return Err(EnvError::NotFound.into());
        }
        self.quartz
            .state()
            .set(StateField::Env, &requested_env.name)?;

        Ok(requested_env)
    }
    pub fn remove_env(&self, name: String) -> Result {
        let env = EnvRef::new(self.quartz, name)?;

        if !env.exists() {
            return Err(EnvError::NotFound.into());
        }
        let curr_env = self.env()?;
        if env.name == curr_env.name {
            return Err(EnvError::EnvInUse(env.name).into());
        }
        env.clean()?;

        Ok(())
    }

    pub fn cp_env(&self, src: String, dest: String) -> Result<EnvRef<'_>> {
        let src = EnvRef::new(self.quartz, src)?;
        let mut dest = EnvRef::new(self.quartz, dest)?;

        for (key, value) in src.variables.iter() {
            dest.variables.insert(key.to_string(), value.to_string());
        }

        for (key, value) in src.headers.iter() {
            dest.headers.insert(key.to_string(), value.to_string());
        }

        dest.save()?;

        Ok(dest)
    }
}
