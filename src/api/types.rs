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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EpisodesResponse {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub count: u32,
    #[serde(default, alias = "episodes")]
    pub items: Vec<Episode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EpisodeResponse {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub description: String,
    #[serde(default, alias = "episode")]
    pub item: Option<Episode>,
    #[serde(default, alias = "episodes")]
    pub items: Vec<Episode>,
}

impl EpisodeResponse {
    pub fn first_episode(&self) -> Option<&Episode> {
        self.item.as_ref().or_else(|| self.items.first())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrendingEpisodesResponse {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub count: u32,
    #[serde(default, alias = "episodes")]
    pub items: Vec<Episode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecentEpisodesResponse {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub count: u32,
    #[serde(default, alias = "episodes")]
    pub items: Vec<Episode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecentFeedsResponse {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub count: u32,
    #[serde(default, alias = "items")]
    pub feeds: Vec<Podcast>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CategoriesResponse {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub count: u32,
    #[serde(default, rename = "feedCount")]
    pub feed_count: Option<u64>,
    #[serde(default, alias = "feeds")]
    pub categories: Vec<Category>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Category {
    #[serde(default)]
    pub id: Option<u64>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatsResponse {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub stats: Stats,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Stats {
    #[serde(default, rename = "feedCountTotal")]
    pub feed_count_total: Option<u64>,
    #[serde(default, rename = "episodeCountTotal")]
    pub episode_count_total: Option<u64>,
    #[serde(default, rename = "feedsWithNewEpisodes3days")]
    pub feeds_with_new_episodes_3days: Option<u64>,
    #[serde(default, rename = "feedsWithNewEpisodes10days")]
    pub feeds_with_new_episodes_10days: Option<u64>,
    #[serde(default, rename = "feedsWithNewEpisodes30days")]
    pub feeds_with_new_episodes_30days: Option<u64>,
    #[serde(default, rename = "feedsWithValueBlocks")]
    pub feeds_with_value_blocks: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Episode {
    #[serde(default)]
    pub id: Option<u64>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub link: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "datePublished")]
    pub date_published: Option<i64>,
    #[serde(default, rename = "datePublishedPretty")]
    pub date_published_pretty: Option<String>,
    #[serde(default)]
    pub duration: Option<u32>,
    #[serde(default, rename = "enclosureUrl")]
    pub enclosure_url: Option<String>,
    #[serde(default, rename = "enclosureType")]
    pub enclosure_type: Option<String>,
    #[serde(default, rename = "enclosureLength")]
    pub enclosure_length: Option<u64>,
    #[serde(default, rename = "feedId")]
    pub feed_id: Option<u64>,
    #[serde(default, rename = "feedTitle")]
    pub feed_title: Option<String>,
    #[serde(default, rename = "feedLanguage")]
    pub feed_language: Option<String>,
    #[serde(default, rename = "feedImage")]
    pub feed_image: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
}
