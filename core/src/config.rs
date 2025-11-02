use serde::{Deserialize, Serialize};
use std::{fs::OpenOptions, io::Write, path::PathBuf};

use crate::{QuartzError, QuartzResult};

pub struct ConfigManager {
    mount_path: PathBuf,
}

impl ConfigManager {
    pub fn new(mount_path: PathBuf) -> Self {
        Self { mount_path }
    }
    pub fn parse(&self) -> Config {
        let parsed = Config::parse(self.mount_path.clone());
        parsed
    }
    pub fn save(&self, mut conf: Config) {
        let save_filepath = Config::filepath(self.mount_path.clone());
        conf.write(save_filepath).unwrap();
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Config {
    pub preferences: Preferences,
    pub ui: UiConfig,
}

impl Config {
    pub fn filename() -> String {
        ".quartz.toml".to_string()
    }

    pub fn filepath(mount_path: PathBuf) -> PathBuf {
        mount_path.join(Self::filename())
    }

    pub fn parse(mount_path: PathBuf) -> Self {
        let filepath = Config::filepath(mount_path);

        if let Ok(config_toml) = std::fs::read_to_string(filepath) {
            return toml::from_str::<Config>(&config_toml).unwrap_or_default();
        }

        Config::default()
    }

    pub fn write(&mut self, file_path: PathBuf) -> QuartzResult {
        let content = toml::to_string(self).map_err(|_| QuartzError::Internal)?;

        if !file_path.exists() {
            let parent_path = file_path.parent().ok_or(QuartzError::Internal)?;
            std::fs::create_dir_all(parent_path).map_err(|_| QuartzError::Internal)?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_path)
            .map_err(|_| QuartzError::Internal)?;

        file.write_all(content.as_bytes())
            .map_err(|_| QuartzError::Internal)?;

        Ok(())
    }
}

#[derive(Serialize, Default, Deserialize)]
pub struct Preferences {
    editor: Option<String>,
    pager: Option<String>,
}

impl Preferences {
    pub fn editor(&self) -> String {
        if let Some(editor) = &self.editor {
            editor.to_owned()
        } else if let Ok(editor) = std::env::var("EDITOR") {
            editor
        } else {
            "vim".to_string()
        }
    }

    pub fn set_editor<T>(&mut self, editor: T)
    where
        T: Into<String>,
    {
        self.editor = Some(editor.into());
    }

    pub fn pager(&self) -> String {
        if let Some(pager) = &self.pager {
            pager.to_owned()
        } else if let Ok(pager) = std::env::var("PAGER") {
            pager.to_string()
        } else {
            "less".to_string()
        }
    }

    pub fn set_pager<T>(&mut self, pager: T)
    where
        T: Into<String>,
    {
        self.pager = Some(pager.into());
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct UiConfig {
    colors: Option<bool>,
}

impl UiConfig {
    pub fn colors(&self) -> bool {
        if std::env::var("NO_COLOR").is_ok() {
            false
        } else if let Ok(clicolor) = std::env::var("CLICOLOR_FORCE") {
            clicolor == "0"
        } else if let Ok(clicolor) = std::env::var("CLICOLOR") {
            clicolor == "0"
        } else if let Some(colors) = self.colors {
            colors
        } else {
            true
        }
    }

    pub fn set_colors(&mut self, colors: bool) {
        self.colors = Some(colors);
    }
}
