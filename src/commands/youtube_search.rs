use std::io::ErrorKind;
use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::cli::YoutubeSearchArgs;
use crate::commands::youtube_meta::{
    ensure_yt_dlp_available, fetch_meta_by_video_id_with_timeout, YT_DLP_BINARY,
};
use crate::error::{PodcastCliError, Result};
use crate::output::json::to_pretty_json;

const YT_DLP_PRINT_TEMPLATE: &str = "%(id)s\t%(title)s\t%(channel)s\t%(duration)s\t%(upload_date)s";
const DEFAULT_META_CONCURRENCY: u8 = 2;
const DEFAULT_META_TIMEOUT_SECONDS: u64 = 15;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct YoutubeSearchItem {
    pub video_id: String,
    pub title: String,
    pub channel: String,
    pub duration: Option<u64>,
    pub upload_date: Option<String>,
    pub url: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct YoutubeSearchItemWithMeta {
    pub video_id: String,
    pub title: String,
    pub channel: String,
    pub duration: Option<u64>,
    pub upload_date: Option<String>,
    pub url: String,
    pub timestamp: Option<i64>,
    pub view_count: Option<u64>,
    pub like_count: Option<u64>,
    pub comment_count: Option<u64>,
    pub availability: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct YoutubeSearchMetaFields {
    duration: Option<u64>,
    upload_date: Option<String>,
    timestamp: Option<i64>,
    view_count: Option<u64>,
    like_count: Option<u64>,
    comment_count: Option<u64>,
    availability: Option<String>,
}

pub async fn run(args: YoutubeSearchArgs) -> Result<()> {
    ensure_yt_dlp_available()?;

    let limit = args.limit.unwrap_or(10);
    validate_limit(limit)?;

    let should_expand_fetch = args.since.is_some() || args.channel.is_some();
    let mut items = fetch_items(
        &args.query,
        limit,
        args.since.as_deref(),
        should_expand_fetch,
    )?;

    if let Some(channel_filter) = args.channel.as_deref() {
        let needle = channel_filter.to_ascii_lowercase();
        items.retain(|item| item.channel.to_ascii_lowercase().contains(&needle));
    }

    items.truncate(limit as usize);

    if should_fetch_meta(&args) {
        let concurrency = args.meta_concurrency.unwrap_or(DEFAULT_META_CONCURRENCY) as usize;
        let timeout_seconds = args.meta_timeout.unwrap_or(DEFAULT_META_TIMEOUT_SECONDS);
        let enriched = enrich_items_with_meta(items, concurrency, timeout_seconds).await;
        println!("{}", to_pretty_json(&enriched)?);
    } else {
        println!("{}", to_pretty_json(&items)?);
    }

    Ok(())
}

fn validate_limit(limit: u32) -> Result<()> {
    if (1..=100).contains(&limit) {
        Ok(())
    } else {
        Err(PodcastCliError::Validation(
            "limit must be in range 1..=100".to_string(),
        ))
    }
}

fn should_fetch_meta(args: &YoutubeSearchArgs) -> bool {
    args.with_meta || args.meta_concurrency.is_some() || args.meta_timeout.is_some()
}

async fn enrich_items_with_meta(
    items: Vec<YoutubeSearchItem>,
    concurrency: usize,
    timeout_seconds: u64,
) -> Vec<YoutubeSearchItemWithMeta> {
    if items.is_empty() {
        return Vec::new();
    }

    let semaphore = Arc::new(Semaphore::new(concurrency.max(1)));
    let mut join_set = JoinSet::new();

    for (index, item) in items.iter().enumerate() {
        let semaphore = Arc::clone(&semaphore);
        let video_id = item.video_id.clone();
        join_set.spawn(async move {
            // Keep permit alive for the full task to enforce concurrency limit.
            let _permit = semaphore
                .acquire_owned()
                .await
                .expect("semaphore should remain available while tasks run");
            let meta = fetch_meta_by_video_id_with_timeout(
                &video_id,
                Some(Duration::from_secs(timeout_seconds)),
            )
            .await
            .ok()
            .map(YoutubeSearchMetaFields::from_meta)
            .unwrap_or_default();

            (index, meta)
        });
    }

    let mut meta_by_index = vec![YoutubeSearchMetaFields::default(); items.len()];
    while let Some(join_result) = join_set.join_next().await {
        if let Ok((index, meta)) = join_result {
            if index < meta_by_index.len() {
                meta_by_index[index] = meta;
            }
        }
    }

    items
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let meta = meta_by_index.get(index).cloned().unwrap_or_default();
            merge_item_with_meta(item, meta)
        })
        .collect()
}

fn merge_item_with_meta(
    item: YoutubeSearchItem,
    meta: YoutubeSearchMetaFields,
) -> YoutubeSearchItemWithMeta {
    YoutubeSearchItemWithMeta {
        video_id: item.video_id,
        title: item.title,
        channel: item.channel,
        duration: meta.duration.or(item.duration),
        upload_date: meta.upload_date.or(item.upload_date),
        url: item.url,
        timestamp: meta.timestamp,
        view_count: meta.view_count,
        like_count: meta.like_count,
        comment_count: meta.comment_count,
        availability: meta.availability,
    }
}

fn fetch_items(
    query: &str,
    limit: u32,
    since: Option<&str>,
    should_expand_fetch: bool,
) -> Result<Vec<YoutubeSearchItem>> {
    let fetch_limit = requested_fetch_limit(limit, should_expand_fetch);

    let mut command = Command::new(YT_DLP_BINARY);
    command.args([
        "--flat-playlist",
        "--no-warnings",
        "--print",
        YT_DLP_PRINT_TEMPLATE,
    ]);

    if let Some(since_arg) = since {
        command.arg("--dateafter").arg(since_arg);
    }

    command.arg(format!("ytsearch{fetch_limit}:{query}"));

    let output = command.output().map_err(|err| {
        if err.kind() == ErrorKind::NotFound {
            PodcastCliError::Config("yt-dlp not found in PATH; install yt-dlp first".to_string())
        } else {
            PodcastCliError::Io(err)
        }
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if stderr.is_empty() {
            "yt-dlp search failed".to_string()
        } else {
            format!("yt-dlp search failed: {stderr}")
        };

        return Err(PodcastCliError::Api(message));
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(parse_row)
        .collect())
}

fn requested_fetch_limit(limit: u32, since_enabled: bool) -> u32 {
    if since_enabled {
        limit.saturating_mul(3).min(200)
    } else {
        limit
    }
}

fn parse_row(line: &str) -> Option<YoutubeSearchItem> {
    let mut parts = line.splitn(5, '\t');

    let video_id = parts.next()?.trim();
    let title = parts.next()?.trim();
    let channel = parts.next()?.trim();
    let duration_raw = parts.next()?.trim();
    let upload_date_raw = parts.next()?.trim();

    if video_id.is_empty() {
        return None;
    }

    let duration = duration_raw.parse::<u64>().ok();
    let upload_date = normalize_upload_date(upload_date_raw);

    Some(YoutubeSearchItem {
        video_id: video_id.to_string(),
        title: title.to_string(),
        channel: channel.to_string(),
        duration,
        upload_date,
        url: format!("https://www.youtube.com/watch?v={video_id}"),
    })
}

fn normalize_upload_date(raw: &str) -> Option<String> {
    if raw.is_empty() {
        return None;
    }

    if raw.len() == 8 && raw.chars().all(|ch| ch.is_ascii_digit()) {
        return Some(format!("{}-{}-{}", &raw[0..4], &raw[4..6], &raw[6..8]));
    }

    Some(raw.to_string())
}

impl YoutubeSearchMetaFields {
    fn from_meta(item: crate::commands::youtube_meta::YoutubeMetaItem) -> Self {
        Self {
            duration: item.duration,
            upload_date: item.upload_date,
            timestamp: item.timestamp,
            view_count: item.view_count,
            like_count: item.like_count,
            comment_count: item.comment_count,
            availability: item.availability,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        merge_item_with_meta, normalize_upload_date, parse_row, requested_fetch_limit,
        YoutubeSearchItem, YoutubeSearchMetaFields,
    };

    #[test]
    fn parse_row_to_item() {
        let row = "dQw4w9WgXcQ\tNever Gonna Give You Up\tRick Astley\t213\t19871025";
        let item = parse_row(row).expect("row should parse");

        assert_eq!(item.video_id, "dQw4w9WgXcQ");
        assert_eq!(item.title, "Never Gonna Give You Up");
        assert_eq!(item.channel, "Rick Astley");
        assert_eq!(item.duration, Some(213));
        assert_eq!(item.upload_date.as_deref(), Some("1987-10-25"));
        assert_eq!(item.url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    }

    #[test]
    fn parse_row_rejects_missing_video_id() {
        assert!(parse_row("\ttitle\tchannel\t60\t20250101").is_none());
    }

    #[test]
    fn parse_row_allows_pipe_in_title() {
        let row = "abc123\tA | B title\tChannel\t60\t20250101";
        let item = parse_row(row).expect("row should parse");
        assert_eq!(item.title, "A | B title");
    }

    #[test]
    fn normalize_upload_date_keeps_non_standard_values() {
        assert_eq!(normalize_upload_date("unknown").as_deref(), Some("unknown"));
        assert_eq!(normalize_upload_date(""), None);
    }

    #[test]
    fn requested_fetch_limit_expands_with_since_filter() {
        assert_eq!(requested_fetch_limit(10, false), 10);
        assert_eq!(requested_fetch_limit(10, true), 30);
        assert_eq!(requested_fetch_limit(100, true), 200);
    }

    #[test]
    fn merge_item_with_meta_prefers_meta_duration_and_upload_date() {
        let item = YoutubeSearchItem {
            video_id: "abc123def45".to_string(),
            title: "Title".to_string(),
            channel: "Channel".to_string(),
            duration: Some(30),
            upload_date: Some("2020-01-01".to_string()),
            url: "https://www.youtube.com/watch?v=abc123def45".to_string(),
        };
        let meta = YoutubeSearchMetaFields {
            duration: Some(120),
            upload_date: Some("2026-03-01".to_string()),
            timestamp: Some(1_772_380_800),
            view_count: Some(1_000),
            like_count: Some(100),
            comment_count: Some(10),
            availability: Some("public".to_string()),
        };

        let merged = merge_item_with_meta(item, meta);
        assert_eq!(merged.duration, Some(120));
        assert_eq!(merged.upload_date.as_deref(), Some("2026-03-01"));
        assert_eq!(merged.view_count, Some(1_000));
    }

    #[test]
    fn merge_item_with_meta_preserves_search_fields_when_meta_missing() {
        let item = YoutubeSearchItem {
            video_id: "abc123def45".to_string(),
            title: "Title".to_string(),
            channel: "Channel".to_string(),
            duration: Some(30),
            upload_date: Some("2020-01-01".to_string()),
            url: "https://www.youtube.com/watch?v=abc123def45".to_string(),
        };

        let merged = merge_item_with_meta(item, YoutubeSearchMetaFields::default());
        assert_eq!(merged.duration, Some(30));
        assert_eq!(merged.upload_date.as_deref(), Some("2020-01-01"));
        assert_eq!(merged.view_count, None);
    }
}
