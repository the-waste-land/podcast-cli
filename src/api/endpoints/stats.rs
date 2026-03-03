use crate::api::client::PodcastIndexClient;
use crate::api::types::StatsResponse;
use crate::error::Result;

pub async fn get_stats(client: &PodcastIndexClient) -> Result<StatsResponse> {
    client.get_json("/stats/current", &[]).await
}
