use crate::api::client::PodcastIndexClient;
use crate::api::types::SearchResponse;
use crate::error::Result;

pub async fn search_by_term(
    client: &PodcastIndexClient,
    term: &str,
    limit: u32,
    music: bool,
) -> Result<SearchResponse> {
    let mut query = vec![("q", term.to_string()), ("max", limit.to_string())];
    if music {
        query.push(("cat", "music".to_string()));
    }

    client.get_json("/search/byterm", &query).await
}

pub async fn search_by_person(
    client: &PodcastIndexClient,
    person: &str,
    limit: u32,
    music: bool,
) -> Result<SearchResponse> {
    let mut query = vec![("q", person.to_string()), ("max", limit.to_string())];
    if music {
        query.push(("cat", "music".to_string()));
    }

    client.get_json("/search/byperson", &query).await
}
