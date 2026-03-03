pub mod json;
pub mod table;

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::PodcastCliError;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Json,
    Table,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Table => write!(f, "table"),
        }
    }
}

impl FromStr for OutputFormat {
    type Err = PodcastCliError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "json" => Ok(Self::Json),
            "table" => Ok(Self::Table),
            _ => Err(PodcastCliError::Validation(format!(
                "unsupported output format: {value}"
            ))),
        }
    }
}
