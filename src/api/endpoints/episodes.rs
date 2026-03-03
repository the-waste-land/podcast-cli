use crate::api::client::PodcastIndexClient;
use crate::api::types::{EpisodeResponse, EpisodesResponse, TrendingEpisodesResponse};
use crate::error::{PodcastCliError, Result};

pub async fn get_episodes_by_feed_id(
    client: &PodcastIndexClient,
    feed_id: u64,
    max: u32,
) -> Result<EpisodesResponse> {
    validate_max(max)?;

    let query = vec![("id", feed_id.to_string()), ("max", max.to_string())];
    client.get_json("/episodes/byfeedid", &query).await
}

pub async fn get_episode_by_id(
    client: &PodcastIndexClient,
    episode_id: u64,
) -> Result<EpisodeResponse> {
    let query = vec![("id", episode_id.to_string())];
    client.get_json("/episodes/byid", &query).await
}

pub async fn get_trending_episodes(
    client: &PodcastIndexClient,
    max: u32,
    lang: Option<&str>,
) -> Result<TrendingEpisodesResponse> {
    validate_max(max)?;

    let mut query = vec![("max", max.to_string())];
    if let Some(lang) = lang.filter(|value| !value.trim().is_empty()) {
        query.push(("lang", lang.to_string()));
    }

    client.get_json("/episodes/trending", &query).await
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
