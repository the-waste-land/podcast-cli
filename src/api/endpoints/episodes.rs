use crate::api::client::PodcastIndexClient;
use crate::api::types::{EpisodeResponse, EpisodesResponse, TrendingEpisodesResponse};
use crate::api::validation::validate_max;
use crate::error::Result;

const TRENDING_EPISODES_PATH: &str = "/episodes/trending";

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

    client.get_json(TRENDING_EPISODES_PATH, &query).await
}


#[cfg(test)]
mod tests {
    #[test]
    fn trending_episodes_endpoint_matches_api_route() {
        assert_eq!(super::TRENDING_EPISODES_PATH, "/episodes/trending");
    }
}
