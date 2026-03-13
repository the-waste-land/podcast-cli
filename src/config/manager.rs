use std::fs;
use std::path::PathBuf;

use crate::config::AppConfig;
use crate::error::{ConfigContext, Result};

const APP_NAME: &str = "podcast-cli";
const CONFIG_NAME: &str = "default";

#[derive(Debug, Clone)]
pub struct ConfigManager {
    path_override: Option<PathBuf>,
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigManager {
    pub fn new() -> Self {
        Self {
            path_override: std::env::var("PODCAST_CLI_CONFIG_PATH")
                .ok()
                .map(PathBuf::from),
        }
    }

    pub fn with_path(path: PathBuf) -> Self {
        Self {
            path_override: Some(path),
        }
    }

    pub fn load(&self) -> Result<AppConfig> {
        if let Some(path) = &self.path_override {
            if !path.exists() {
                return Ok(AppConfig::default());
            }

            return confy::load_path(path).config_context("failed to load config");
        }

        confy::load(APP_NAME, Some(CONFIG_NAME)).config_context("failed to load config")
    }

    pub fn save(&self, cfg: &AppConfig) -> Result<()> {
        if let Some(path) = &self.path_override {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            return confy::store_path(path, cfg).config_context("failed to save config");
        }

        confy::store(APP_NAME, Some(CONFIG_NAME), cfg).config_context("failed to save config")
    }

    pub fn clear(&self) -> Result<()> {
        let path = if let Some(path) = &self.path_override {
            path.clone()
        } else {
            confy::get_configuration_file_path(APP_NAME, Some(CONFIG_NAME))
                .config_context("failed to resolve config path")?
        };

        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}
