use std::io::ErrorKind;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

use crate::cli::{OutputArg, YoutubeMetaArgs};
use crate::error::{PodcastCliError, Result};
use crate::output::json::to_pretty_json;
use crate::output::table::render_youtube_meta;

pub const YT_DLP_BINARY: &str = "yt-dlp";

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct YoutubeMetaItem {
    pub video_id: String,
    pub title: Option<String>,
    pub channel: Option<String>,
    pub url: String,
    pub duration: Option<u64>,
    pub upload_date: Option<String>,
    pub timestamp: Option<i64>,
    pub view_count: Option<u64>,
    pub like_count: Option<u64>,
    pub comment_count: Option<u64>,
    pub availability: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawYoutubeMetaItem {
    id: Option<String>,
    title: Option<String>,
    channel: Option<String>,
    uploader: Option<String>,
    webpage_url: Option<String>,
    duration: Option<u64>,
    upload_date: Option<String>,
    timestamp: Option<i64>,
    view_count: Option<u64>,
    like_count: Option<u64>,
    comment_count: Option<u64>,
    availability: Option<String>,
}

pub async fn run(args: YoutubeMetaArgs) -> Result<()> {
    ensure_yt_dlp_available()?;
    let item = fetch_meta_by_video_id(&args.video_id).await?;

    let rendered = match args.output.unwrap_or(OutputArg::Json) {
        OutputArg::Json => to_pretty_json(&item)?,
        OutputArg::Table => render_youtube_meta(&item),
    };

    println!("{rendered}");
    Ok(())
}

pub fn ensure_yt_dlp_available() -> Result<()> {
    match std::process::Command::new(YT_DLP_BINARY)
        .arg("--version")
        .output()
    {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let message = if stderr.trim().is_empty() {
                "yt-dlp is installed but not executable".to_string()
            } else {
                format!(
                    "yt-dlp is installed but unavailable: {}",
                    stderr.trim().replace('\n', " ")
                )
            };
            Err(PodcastCliError::Config(message))
        }
        Err(err) if err.kind() == ErrorKind::NotFound => Err(PodcastCliError::Config(
            "yt-dlp not found in PATH; install yt-dlp first".to_string(),
        )),
        Err(err) => Err(PodcastCliError::Io(err)),
    }
}

pub async fn fetch_meta_by_video_id(video_id: &str) -> Result<YoutubeMetaItem> {
    fetch_meta_by_video_id_with_timeout(video_id, None).await
}

pub async fn fetch_meta_by_video_id_with_timeout(
    video_id: &str,
    timeout: Option<Duration>,
) -> Result<YoutubeMetaItem> {
    let url = format!("https://www.youtube.com/watch?v={video_id}");
    let mut command = Command::new(YT_DLP_BINARY);
    command
        .args(["--skip-download", "--no-warnings", "--dump-single-json"])
        .arg(url);

    let output = match timeout {
        Some(duration) => {
            let child = command.spawn().map_err(|err| {
                if err.kind() == ErrorKind::NotFound {
                    PodcastCliError::Config(
                        "yt-dlp not found in PATH; install yt-dlp first".to_string(),
                    )
                } else {
                    PodcastCliError::Io(err)
                }
            })?;
            match tokio::time::timeout(duration, child.wait_with_output()).await {
                Ok(Ok(output)) => output,
                Ok(Err(err)) => {
                    return Err(PodcastCliError::Io(err));
                }
                Err(_) => {
                    return Err(PodcastCliError::Api(format!(
                        "yt-dlp metadata request timed out for video-id `{video_id}` after {}s",
                        duration.as_secs()
                    )));
                }
            }
        }
        None => command.output().await.map_err(|err| {
            if err.kind() == ErrorKind::NotFound {
                PodcastCliError::Config(
                    "yt-dlp not found in PATH; install yt-dlp first".to_string(),
                )
            } else {
                PodcastCliError::Io(err)
            }
        })?,
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if stderr.is_empty() {
            format!("yt-dlp failed for video-id `{video_id}`")
        } else {
            format!("yt-dlp failed for video-id `{video_id}`: {stderr}")
        };
        return Err(PodcastCliError::Api(message));
    }

    let raw = serde_json::from_slice::<RawYoutubeMetaItem>(&output.stdout)
        .map_err(PodcastCliError::Serialization)?;
    Ok(map_raw_meta_item(video_id, raw))
}

fn map_raw_meta_item(video_id: &str, raw: RawYoutubeMetaItem) -> YoutubeMetaItem {
    let normalized_video_id =
        normalize_optional_text(raw.id).unwrap_or_else(|| video_id.to_string());
    let url = normalize_optional_text(raw.webpage_url)
        .unwrap_or_else(|| format!("https://www.youtube.com/watch?v={normalized_video_id}"));

    YoutubeMetaItem {
        video_id: normalized_video_id,
        title: normalize_optional_text(raw.title),
        channel: normalize_optional_text(raw.channel)
            .or_else(|| normalize_optional_text(raw.uploader)),
        url,
        duration: raw.duration,
        upload_date: normalize_upload_date(raw.upload_date.as_deref()),
        timestamp: raw.timestamp,
        view_count: raw.view_count,
        like_count: raw.like_count,
        comment_count: raw.comment_count,
        availability: normalize_optional_text(raw.availability),
    }
}

fn normalize_upload_date(raw: Option<&str>) -> Option<String> {
    let trimmed = raw?.trim();
    if trimmed.is_empty() {
        return None;
    }

    if trimmed.eq_ignore_ascii_case("na") || trimmed.eq_ignore_ascii_case("null") {
        return None;
    }

    if trimmed.len() == 8 && trimmed.chars().all(|ch| ch.is_ascii_digit()) {
        return Some(format!(
            "{}-{}-{}",
            &trimmed[0..4],
            &trimmed[4..6],
            &trimmed[6..8]
        ));
    }

    Some(trimmed.to_string())
}

fn normalize_optional_text(value: Option<String>) -> Option<String> {
    let value = value?;
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{map_raw_meta_item, normalize_upload_date, RawYoutubeMetaItem};

    #[test]
    fn normalize_upload_date_formats_yyyymmdd() {
        assert_eq!(
            normalize_upload_date(Some("20260301")).as_deref(),
            Some("2026-03-01")
        );
    }

    #[test]
    fn normalize_upload_date_keeps_non_standard() {
        assert_eq!(
            normalize_upload_date(Some("2026-03-01")).as_deref(),
            Some("2026-03-01")
        );
        assert_eq!(normalize_upload_date(Some("  ")), None);
        assert_eq!(normalize_upload_date(Some("NA")), None);
        assert_eq!(normalize_upload_date(Some("na")), None);
        assert_eq!(normalize_upload_date(Some("null")), None);
        assert_eq!(normalize_upload_date(Some("NULL")), None);
        assert_eq!(normalize_upload_date(None), None);
    }

    #[test]
    fn map_raw_meta_item_applies_fallbacks() {
        let raw = RawYoutubeMetaItem {
            id: None,
            title: Some("  A Talk  ".to_string()),
            channel: None,
            uploader: Some("  Host  ".to_string()),
            webpage_url: None,
            duration: Some(123),
            upload_date: Some("20260301".to_string()),
            timestamp: Some(1_772_380_800),
            view_count: Some(1_000),
            like_count: None,
            comment_count: Some(50),
            availability: Some("public".to_string()),
        };

        let item = map_raw_meta_item("abc123def45", raw);
        assert_eq!(item.video_id, "abc123def45");
        assert_eq!(item.title.as_deref(), Some("A Talk"));
        assert_eq!(item.channel.as_deref(), Some("Host"));
        assert_eq!(
            item.url,
            "https://www.youtube.com/watch?v=abc123def45".to_string()
        );
        assert_eq!(item.upload_date.as_deref(), Some("2026-03-01"));
    }
}
