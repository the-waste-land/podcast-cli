use crate::api::client::PodcastIndexClient;
use crate::api::types::CategoriesResponse;
use crate::error::Result;

pub async fn get_categories(client: &PodcastIndexClient) -> Result<CategoriesResponse> {
    client.get_json("/categories/list", &[]).await
}
