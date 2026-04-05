use crate::{
    Quartz,
    env::{env_ref::EnvRef, error::EnvError, value::Env},
    error::Result,
    state::field::StateField,
};

pub mod env_ref;
pub mod error;
pub mod value;

impl Quartz {
    pub fn current_env(&self) -> Result<EnvRef<'_>> {
        let curr_env_name = self.state_get(StateField::Env).unwrap_or("default".into());

        let parsed_env = EnvRef::new(self, curr_env_name);
        Ok(parsed_env)
    }
    pub fn get_env(&self, name: String) -> Option<EnvRef<'_>> {
        let env = EnvRef::new(self, name);
        if env.exists() { Some(env) } else { None }
    }
    pub fn create_env(&self, name: String) -> Result<EnvRef<'_>> {
        let new_env = EnvRef::new(self, name);

        new_env.save(&Env::default())?;

        Ok(new_env)
    }
    pub fn envs(&self) -> Result<impl Iterator<Item = Result<String>>> {
        let entries = std::fs::read_dir(self.path().join("env")).map_err(EnvError::GetEnvs)?;
        let env_names = entries.map(|entry| {
            let ok_dir_entry = entry.map_err(EnvError::GetEnvs)?;
            let filename = ok_dir_entry.file_name();
            let str_filename = filename.to_str().ok_or(EnvError::ParseEnvName)?.to_owned();
            Ok(str_filename)
        });

        Ok(env_names)
    }
    pub fn switch_env(&self, name: String) -> Result<EnvRef<'_>> {
        let requested_env = EnvRef::new(self, name);
        if !requested_env.exists() {
            return Err(EnvError::NotFound.into());
        }
        self.state_set(StateField::Env, &requested_env.name)?;

        Ok(requested_env)
    }
    pub fn remove_env(&self, name: String) -> Result {
        let env = EnvRef::new(self, name);

        if !env.exists() {
            return Err(EnvError::NotFound.into());
        }
        let curr_env = self.current_env()?;
        if env.name == curr_env.name {
            return Err(EnvError::EnvInUse(env.name).into());
        }
        env.clean()?;

        Ok(())
    }

    pub fn copy_env(&self, src: String, dest: String) -> Result<EnvRef<'_>> {
        let src_ref = EnvRef::new(self, src);
        let src = src_ref.read()?;
        let dest_ref = EnvRef::new(self, dest);

        dest_ref.save(&src)?;

        Ok(dest_ref)
    }
}
