use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchResponse {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub count: u32,
    #[serde(default)]
    pub feeds: Vec<Podcast>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PodcastResponse {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub count: u32,
    #[serde(default)]
    pub feed: Option<Podcast>,
    #[serde(default)]
    pub feeds: Vec<Podcast>,
}

impl PodcastResponse {
    pub fn first_podcast(&self) -> Option<&Podcast> {
        self.feed.as_ref().or_else(|| self.feeds.first())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Podcast {
    #[serde(default)]
    pub id: u64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub language: String,
    #[serde(default, rename = "feedUrl")]
    pub feed_url: String,
    #[serde(default, rename = "url")]
    pub website: String,
    #[serde(default)]
    pub description: String,
}
