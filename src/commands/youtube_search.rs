use std::io::ErrorKind;
use std::process::Command;

use serde::Serialize;

use crate::cli::YoutubeSearchArgs;
use crate::error::{PodcastCliError, Result};
use crate::output::json::to_pretty_json;

const YT_DLP_BINARY: &str = "yt-dlp";
const YT_DLP_PRINT_TEMPLATE: &str = "%(id)s\t%(title)s\t%(channel)s\t%(duration)s\t%(upload_date)s";

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct YoutubeSearchItem {
    pub video_id: String,
    pub title: String,
    pub channel: String,
    pub duration: Option<u64>,
    pub upload_date: Option<String>,
    pub url: String,
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

    println!("{}", to_pretty_json(&items)?);
    Ok(())
}

fn ensure_yt_dlp_available() -> Result<()> {
    match Command::new(YT_DLP_BINARY).arg("--version").output() {
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

fn validate_limit(limit: u32) -> Result<()> {
    if (1..=100).contains(&limit) {
        Ok(())
    } else {
        Err(PodcastCliError::Validation(
            "limit must be in range 1..=100".to_string(),
        ))
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

#[cfg(test)]
mod tests {
    use super::{normalize_upload_date, parse_row, requested_fetch_limit};

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
}
