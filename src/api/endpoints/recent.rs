use crate::api::client::PodcastIndexClient;
use crate::api::types::{RecentEpisodesResponse, RecentFeedsResponse};
use crate::error::{PodcastCliError, Result};

pub async fn get_recent_episodes(
    client: &PodcastIndexClient,
    max: Option<u32>,
    before: Option<i64>,
) -> Result<RecentEpisodesResponse> {
    if let Some(limit) = max {
        validate_max(limit)?;
    }
    if let Some(value) = before {
        validate_timestamp("before", value)?;
    }

    let mut query = Vec::new();
    if let Some(limit) = max {
        query.push(("max", limit.to_string()));
    }
    if let Some(value) = before {
        query.push(("before", value.to_string()));
    }

    client.get_json("/recent/episodes", &query).await
}

pub async fn get_recent_feeds(
    client: &PodcastIndexClient,
    max: Option<u32>,
    since: Option<i64>,
) -> Result<RecentFeedsResponse> {
    if let Some(limit) = max {
        validate_max(limit)?;
    }
    if let Some(value) = since {
        validate_timestamp("since", value)?;
    }

    let mut query = Vec::new();
    if let Some(limit) = max {
        query.push(("max", limit.to_string()));
    }
    if let Some(value) = since {
        query.push(("since", value.to_string()));
    }

    client.get_json("/recent/feeds", &query).await
}

fn validate_max(max: u32) -> Result<()> {
    if (1..=100).contains(&max) {
        Ok(())
    } else {
        Err(PodcastCliError::Validation(
            "limit must be in range 1..=100".to_string(),
        ))
    }
}

fn validate_timestamp(label: &str, value: i64) -> Result<()> {
    if value >= 0 {
        Ok(())
    } else {
        Err(PodcastCliError::Validation(format!(
            "{label} must be a non-negative unix timestamp"
        )))
    }
}
