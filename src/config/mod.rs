mod manager;

use serde::{Deserialize, Serialize};

use crate::error::{PodcastCliError, Result};
use crate::output::OutputFormat;

pub use manager::ConfigManager;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub default_output: OutputFormat,
    pub max_results: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_secret: None,
            default_output: OutputFormat::Table,
            max_results: 10,
        }
    }
}

impl AppConfig {
    pub fn require_credentials(&self) -> Result<(&str, &str)> {
        let key = self
            .api_key
            .as_deref()
            .ok_or_else(|| PodcastCliError::Config("api_key is not configured".to_string()))?;
        let secret = self
            .api_secret
            .as_deref()
            .ok_or_else(|| PodcastCliError::Config("api_secret is not configured".to_string()))?;

        if key.is_empty() || secret.is_empty() {
            return Err(PodcastCliError::Config(
                "api_key/api_secret cannot be empty".to_string(),
            ));
        }

        Ok((key, secret))
    }

    pub fn masked(&self) -> Self {
        Self {
            api_key: mask_sensitive(&self.api_key),
            api_secret: mask_sensitive(&self.api_secret),
            default_output: self.default_output,
            max_results: self.max_results,
        }
    }
}

fn mask_sensitive(value: &Option<String>) -> Option<String> {
    let raw = value.as_deref()?;
    if raw.is_empty() {
        return Some(String::new());
    }

    if raw.len() <= 4 {
        return Some("*".repeat(4));
    }

    let suffix = &raw[raw.len() - 4..];
    Some(format!("{}{}", "*".repeat(raw.len() - 4), suffix))
}
