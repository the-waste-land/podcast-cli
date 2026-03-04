use crate::error::{PodcastCliError, Result};

pub(crate) fn validate_max(max: u32) -> Result<()> {
    if (1..=100).contains(&max) {
        Ok(())
    } else {
        Err(PodcastCliError::Validation(
            "limit must be in range 1..=100".to_string(),
        ))
    }
}

pub(crate) fn validate_timestamp(label: &str, value: i64) -> Result<()> {
    if value >= 0 {
        Ok(())
    } else {
        Err(PodcastCliError::Validation(format!(
            "{label} must be a non-negative unix timestamp"
        )))
    }
}
