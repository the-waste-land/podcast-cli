use crate::api::client::PodcastIndexClient;
use crate::api::types::PodcastResponse;
use crate::error::Result;

pub async fn podcast_by_feed_id(
    client: &PodcastIndexClient,
    feed_id: u64,
) -> Result<PodcastResponse> {
    let query = vec![("id", feed_id.to_string())];
    client.get_json("/podcasts/byfeedid", &query).await
}

pub async fn podcast_by_feed_url(
    client: &PodcastIndexClient,
    feed_url: &str,
) -> Result<PodcastResponse> {
    let query = vec![("url", feed_url.to_string())];
    client.get_json("/podcasts/byfeedurl", &query).await
}
