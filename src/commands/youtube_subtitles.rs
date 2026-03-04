use std::collections::HashMap;
use std::io::ErrorKind;
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::cli::{SubtitleOutputArg, YoutubeSubtitlesArgs};
use crate::error::{PodcastCliError, Result};
use crate::output::json::to_pretty_json;

const YT_DLP_BINARY: &str = "yt-dlp";

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct SubtitleSegment {
    pub index: usize,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct YoutubeSubtitlesResult {
    pub video_id: String,
    pub language: String,
    pub title: String,
    pub segments: Vec<SubtitleSegment>,
    pub text: String,
    pub segment_count: usize,
}

#[derive(Debug, Deserialize)]
struct VideoMetadata {
    #[serde(default)]
    id: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    subtitles: HashMap<String, Vec<TrackEntry>>,
    #[serde(default, rename = "automatic_captions")]
    automatic_captions: HashMap<String, Vec<TrackEntry>>,
}

#[derive(Debug, Deserialize, Clone)]
struct TrackEntry {
    ext: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Clone)]
struct SelectedTrack {
    language: String,
    ext: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct Json3Payload {
    #[serde(default)]
    events: Vec<Json3Event>,
}

#[derive(Debug, Deserialize)]
struct Json3Event {
    #[serde(rename = "tStartMs")]
    t_start_ms: Option<u64>,
    #[serde(rename = "dDurationMs")]
    d_duration_ms: Option<u64>,
    #[serde(default)]
    segs: Vec<Json3Segment>,
}

#[derive(Debug, Deserialize)]
struct Json3Segment {
    #[serde(default)]
    utf8: String,
}

pub async fn run(args: YoutubeSubtitlesArgs) -> Result<()> {
    ensure_yt_dlp_available()?;

    let metadata = fetch_video_metadata(&args.video_id)?;
    let track = select_track_by_lang(&metadata, &args.lang)?;
    let raw_content = download_subtitle_track(&track.url).await?;
    let segments = parse_subtitle_content(&raw_content, &track.ext)?;

    if segments.is_empty() {
        return Err(PodcastCliError::Metadata(
            "subtitle track is empty".to_string(),
        ));
    }

    let normalized_video_id = if metadata.id.trim().is_empty() {
        args.video_id
    } else {
        metadata.id
    };

    let title = if metadata.title.trim().is_empty() {
        normalized_video_id.clone()
    } else {
        metadata.title
    };

    let text = aggregate_text(&segments);
    let result = YoutubeSubtitlesResult {
        video_id: normalized_video_id,
        language: track.language,
        title,
        segment_count: segments.len(),
        segments,
        text,
    };

    let rendered = match args.output {
        SubtitleOutputArg::Json => to_pretty_json(&result)?,
        SubtitleOutputArg::Text => result.text.clone(),
        SubtitleOutputArg::Srt => render_srt(&result.segments),
    };

    println!("{rendered}");
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

fn fetch_video_metadata(video_id: &str) -> Result<VideoMetadata> {
    let url = format!("https://www.youtube.com/watch?v={video_id}");
    let output = Command::new(YT_DLP_BINARY)
        .args(["--skip-download", "--no-warnings", "--dump-single-json"]) // required contract
        .arg(url)
        .output()
        .map_err(|err| {
            if err.kind() == ErrorKind::NotFound {
                PodcastCliError::Config(
                    "yt-dlp not found in PATH; install yt-dlp first".to_string(),
                )
            } else {
                PodcastCliError::Io(err)
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if stderr.is_empty() {
            format!("yt-dlp failed for video-id `{video_id}`")
        } else {
            format!("yt-dlp failed for video-id `{video_id}`: {stderr}")
        };

        return Err(PodcastCliError::Api(message));
    }

    serde_json::from_slice::<VideoMetadata>(&output.stdout).map_err(PodcastCliError::Serialization)
}

fn select_track_by_lang(metadata: &VideoMetadata, requested_lang: &str) -> Result<SelectedTrack> {
    if let Some(track) = select_track_from_map(&metadata.subtitles, requested_lang) {
        return Ok(track);
    }

    if let Some(track) = select_track_from_map(&metadata.automatic_captions, requested_lang) {
        return Ok(track);
    }

    let mut available = metadata
        .subtitles
        .keys()
        .chain(metadata.automatic_captions.keys())
        .cloned()
        .collect::<Vec<_>>();
    available.sort();
    available.dedup();

    let suffix = if available.is_empty() {
        "no subtitles advertised by yt-dlp".to_string()
    } else {
        format!("available languages: {}", available.join(", "))
    };

    Err(PodcastCliError::Metadata(format!(
        "no subtitle track for language `{requested_lang}`; {suffix}"
    )))
}

fn select_track_from_map(
    tracks_map: &HashMap<String, Vec<TrackEntry>>,
    requested_lang: &str,
) -> Option<SelectedTrack> {
    let mut best: Option<(u8, u8, SelectedTrack)> = None;

    for (language, tracks) in tracks_map {
        let Some(lang_score) = language_score(requested_lang, language) else {
            continue;
        };

        for track in tracks {
            let Some(url) = track.url.as_deref() else {
                continue;
            };

            let ext = infer_track_ext(track, url);
            let ext_score = ext_priority(&ext);
            let candidate = SelectedTrack {
                language: language.clone(),
                ext,
                url: url.to_string(),
            };

            match &best {
                Some((best_lang, _, _)) if *best_lang > lang_score => {}
                Some((best_lang, best_ext, _))
                    if *best_lang == lang_score && *best_ext >= ext_score => {}
                _ => {
                    best = Some((lang_score, ext_score, candidate));
                }
            }
        }
    }

    best.map(|(_, _, track)| track)
}

fn language_score(requested: &str, candidate: &str) -> Option<u8> {
    let req = requested.trim().to_ascii_lowercase();
    let cand = candidate.trim().to_ascii_lowercase();

    if req.is_empty() || cand.is_empty() {
        return None;
    }

    if req == cand {
        return Some(2);
    }

    let prefix_hyphen = format!("{req}-");
    let prefix_underscore = format!("{req}_");

    if cand.starts_with(&prefix_hyphen) || cand.starts_with(&prefix_underscore) {
        Some(1)
    } else {
        None
    }
}

fn infer_track_ext(track: &TrackEntry, url: &str) -> String {
    if let Some(ext) = track.ext.as_deref() {
        let normalized = ext.trim().trim_start_matches('.').to_ascii_lowercase();
        if !normalized.is_empty() {
            return normalized;
        }
    }

    let without_query = url.split('?').next().unwrap_or(url);
    let inferred = without_query
        .rsplit_once('.')
        .map(|(_, ext)| ext.trim().to_ascii_lowercase())
        .unwrap_or_default();

    if inferred.is_empty() {
        "unknown".to_string()
    } else {
        inferred
    }
}

fn ext_priority(ext: &str) -> u8 {
    match ext {
        "json3" => 3,
        "vtt" => 2,
        "srt" => 1,
        _ => 0,
    }
}

async fn download_subtitle_track(url: &str) -> Result<String> {
    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        return Err(PodcastCliError::Api(format!(
            "failed to download subtitle track: HTTP {}",
            response.status()
        )));
    }

    let body = response.text().await?;
    if body.trim().is_empty() {
        return Err(PodcastCliError::Metadata(
            "subtitle track content is empty".to_string(),
        ));
    }

    Ok(body)
}

fn parse_subtitle_content(content: &str, ext: &str) -> Result<Vec<SubtitleSegment>> {
    let normalized_ext = ext.trim().to_ascii_lowercase();

    let mut segments = match normalized_ext.as_str() {
        "json3" => parse_json3_segments(content)?,
        "vtt" => parse_vtt_segments(content)?,
        "srt" => parse_srt_segments(content)?,
        _ => {
            let trimmed = content.trim_start();
            if trimmed.starts_with('{') {
                parse_json3_segments(content)?
            } else if trimmed.starts_with("WEBVTT") {
                parse_vtt_segments(content)?
            } else {
                parse_srt_segments(content)?
            }
        }
    };

    normalize_segments(&mut segments);
    Ok(segments)
}

fn parse_json3_segments(content: &str) -> Result<Vec<SubtitleSegment>> {
    let payload =
        serde_json::from_str::<Json3Payload>(content).map_err(PodcastCliError::Serialization)?;

    let mut segments = Vec::new();
    for event in payload.events {
        let text = normalize_text(
            &event
                .segs
                .iter()
                .map(|segment| segment.utf8.as_str())
                .collect::<Vec<_>>()
                .join(" "),
        );

        if text.is_empty() {
            continue;
        }

        let start_ms = event.t_start_ms.unwrap_or(0);
        let duration_ms = event.d_duration_ms.unwrap_or(1000);
        let end_ms = start_ms.saturating_add(duration_ms.max(1));

        segments.push(SubtitleSegment {
            index: segments.len() + 1,
            start_ms,
            end_ms,
            text,
        });
    }

    if segments.is_empty() {
        return Err(PodcastCliError::Metadata(
            "json3 subtitle payload has no text segments".to_string(),
        ));
    }

    Ok(segments)
}

fn parse_vtt_segments(content: &str) -> Result<Vec<SubtitleSegment>> {
    let lines = content.replace("\r\n", "\n");
    let all_lines = lines.lines().collect::<Vec<_>>();

    let mut segments = Vec::new();
    let mut index = 0;
    while index < all_lines.len() {
        let current = all_lines[index].trim();

        if current.is_empty() || current == "WEBVTT" {
            index += 1;
            continue;
        }

        if current.starts_with("NOTE") {
            index += 1;
            while index < all_lines.len() && !all_lines[index].trim().is_empty() {
                index += 1;
            }
            continue;
        }

        let timing_line = if current.contains("-->") {
            current
        } else if index + 1 < all_lines.len() && all_lines[index + 1].contains("-->") {
            index += 1;
            all_lines[index].trim()
        } else {
            index += 1;
            continue;
        };

        let (start_ms, end_ms) = parse_cue_range(timing_line)?;
        index += 1;

        let mut cue_text = Vec::new();
        while index < all_lines.len() {
            let line = all_lines[index].trim();
            if line.is_empty() {
                break;
            }
            cue_text.push(line);
            index += 1;
        }

        let text = normalize_text(&cue_text.join(" "));
        if !text.is_empty() {
            segments.push(SubtitleSegment {
                index: segments.len() + 1,
                start_ms,
                end_ms,
                text,
            });
        }

        while index < all_lines.len() && !all_lines[index].trim().is_empty() {
            index += 1;
        }
    }

    if segments.is_empty() {
        return Err(PodcastCliError::Metadata(
            "vtt subtitle payload has no text segments".to_string(),
        ));
    }

    Ok(segments)
}

fn parse_srt_segments(content: &str) -> Result<Vec<SubtitleSegment>> {
    let normalized = content.replace("\r\n", "\n");
    let mut segments = Vec::new();

    for block in normalized.split("\n\n") {
        let lines = block
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>();

        if lines.is_empty() {
            continue;
        }

        let mut cursor = 0;
        if lines[cursor].parse::<usize>().is_ok() && lines.len() > 1 {
            cursor += 1;
        }

        if cursor >= lines.len() || !lines[cursor].contains("-->") {
            continue;
        }

        let (start_ms, end_ms) = parse_cue_range(lines[cursor])?;
        let text = normalize_text(&lines[cursor + 1..].join(" "));
        if text.is_empty() {
            continue;
        }

        segments.push(SubtitleSegment {
            index: segments.len() + 1,
            start_ms,
            end_ms,
            text,
        });
    }

    if segments.is_empty() {
        return Err(PodcastCliError::Metadata(
            "srt subtitle payload has no text segments".to_string(),
        ));
    }

    Ok(segments)
}

fn parse_cue_range(line: &str) -> Result<(u64, u64)> {
    let (start_raw, end_raw) = line
        .split_once("-->")
        .ok_or_else(|| PodcastCliError::Metadata(format!("invalid cue timing line: `{line}`")))?;

    let start_ms = parse_timestamp_ms(start_raw.trim())?;
    let end_token = end_raw
        .split_whitespace()
        .next()
        .ok_or_else(|| PodcastCliError::Metadata(format!("invalid cue end timing: `{line}`")))?;
    let end_ms = parse_timestamp_ms(end_token)?;

    Ok((start_ms, end_ms.max(start_ms + 1)))
}

fn parse_timestamp_ms(value: &str) -> Result<u64> {
    let trimmed = value.trim();
    let parts = trimmed.split(':').collect::<Vec<_>>();

    let (hours, minutes, seconds_ms) = match parts.len() {
        3 => (parts[0], parts[1], parts[2]),
        2 => ("0", parts[0], parts[1]),
        _ => {
            return Err(PodcastCliError::Metadata(format!(
                "invalid timestamp: `{value}`"
            )))
        }
    };

    let hours = hours.parse::<u64>().map_err(|_| {
        PodcastCliError::Metadata(format!("invalid timestamp hour component: `{value}`"))
    })?;
    let minutes = minutes.parse::<u64>().map_err(|_| {
        PodcastCliError::Metadata(format!("invalid timestamp minute component: `{value}`"))
    })?;

    let (seconds_raw, millis_raw) = match seconds_ms.split_once('.') {
        Some((sec, ms)) => (sec, ms),
        None => match seconds_ms.split_once(',') {
            Some((sec, ms)) => (sec, ms),
            None => (seconds_ms, "0"),
        },
    };

    let seconds = seconds_raw.parse::<u64>().map_err(|_| {
        PodcastCliError::Metadata(format!("invalid timestamp second component: `{value}`"))
    })?;
    let millis = parse_millis(millis_raw).ok_or_else(|| {
        PodcastCliError::Metadata(format!(
            "invalid timestamp millisecond component: `{value}`"
        ))
    })?;

    Ok((((hours * 60 + minutes) * 60 + seconds) * 1000) + millis)
}

fn parse_millis(raw: &str) -> Option<u64> {
    let digits = raw
        .chars()
        .take_while(|value| value.is_ascii_digit())
        .collect::<String>();

    if digits.is_empty() {
        return Some(0);
    }

    let value = digits.parse::<u64>().ok()?;
    match digits.len() {
        1 => Some(value * 100),
        2 => Some(value * 10),
        _ => Some(value / 10_u64.pow((digits.len() - 3) as u32)),
    }
}

fn normalize_segments(segments: &mut [SubtitleSegment]) {
    for (idx, segment) in segments.iter_mut().enumerate() {
        segment.index = idx + 1;
        segment.text = normalize_text(&segment.text);
        if segment.end_ms <= segment.start_ms {
            segment.end_ms = segment.start_ms + 1;
        }
    }
}

fn normalize_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn aggregate_text(segments: &[SubtitleSegment]) -> String {
    segments
        .iter()
        .map(|segment| segment.text.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_srt(segments: &[SubtitleSegment]) -> String {
    let mut output = String::new();

    for (idx, segment) in segments.iter().enumerate() {
        let line = format!(
            "{}\n{} --> {}\n{}\n",
            idx + 1,
            format_srt_timestamp(segment.start_ms),
            format_srt_timestamp(segment.end_ms),
            segment.text
        );
        output.push_str(&line);

        if idx + 1 < segments.len() {
            output.push('\n');
        }
    }

    output
}

fn format_srt_timestamp(total_ms: u64) -> String {
    let hours = total_ms / 3_600_000;
    let minutes = (total_ms % 3_600_000) / 60_000;
    let seconds = (total_ms % 60_000) / 1000;
    let millis = total_ms % 1000;

    format!("{hours:02}:{minutes:02}:{seconds:02},{millis:03}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(ext: &str, url: &str) -> TrackEntry {
        TrackEntry {
            ext: Some(ext.to_string()),
            url: Some(url.to_string()),
        }
    }

    #[test]
    fn select_track_prefers_manual_subtitles() {
        let metadata = VideoMetadata {
            id: "abc123def45".to_string(),
            title: "video".to_string(),
            subtitles: HashMap::from([(
                "en".to_string(),
                vec![track("vtt", "https://example.com/manual-en.vtt")],
            )]),
            automatic_captions: HashMap::from([(
                "en".to_string(),
                vec![track("json3", "https://example.com/auto-en.json3")],
            )]),
        };

        let selected = select_track_by_lang(&metadata, "en").expect("select track");
        assert_eq!(selected.url, "https://example.com/manual-en.vtt");
        assert_eq!(selected.ext, "vtt");
    }

    #[test]
    fn select_track_supports_language_prefix() {
        let metadata = VideoMetadata {
            id: "abc123def45".to_string(),
            title: "video".to_string(),
            subtitles: HashMap::from([(
                "en-US".to_string(),
                vec![track("json3", "https://example.com/en-us.json3")],
            )]),
            automatic_captions: HashMap::new(),
        };

        let selected = select_track_by_lang(&metadata, "en").expect("select track");
        assert_eq!(selected.language, "en-US");
        assert_eq!(selected.ext, "json3");
    }

    #[test]
    fn select_track_uses_automatic_when_manual_missing() {
        let metadata = VideoMetadata {
            id: "abc123def45".to_string(),
            title: "video".to_string(),
            subtitles: HashMap::new(),
            automatic_captions: HashMap::from([(
                "zh".to_string(),
                vec![track("srt", "https://example.com/zh.srt")],
            )]),
        };

        let selected = select_track_by_lang(&metadata, "zh").expect("select track");
        assert_eq!(selected.language, "zh");
        assert_eq!(selected.ext, "srt");
    }

    #[test]
    fn parse_json3_segments_normalizes_text() {
        let json = r#"{
  "events": [
    {
      "tStartMs": 0,
      "dDurationMs": 1500,
      "segs": [{"utf8": "Hello"}, {"utf8": " world"}]
    },
    {
      "tStartMs": 1500,
      "dDurationMs": 800,
      "segs": [{"utf8": "next line"}]
    }
  ]
}"#;

        let segments = parse_subtitle_content(json, "json3").expect("parse json3");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].start_ms, 0);
        assert_eq!(segments[0].end_ms, 1500);
        assert_eq!(segments[0].text, "Hello world");
        assert_eq!(segments[1].text, "next line");
    }

    #[test]
    fn parse_vtt_segments_to_unified_model() {
        let vtt = "WEBVTT\n\n00:00:00.000 --> 00:00:01.500\nHello world\n\n00:00:01.500 --> 00:00:03.000\nSecond line\n";

        let segments = parse_subtitle_content(vtt, "vtt").expect("parse vtt");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].start_ms, 0);
        assert_eq!(segments[0].end_ms, 1500);
        assert_eq!(segments[1].start_ms, 1500);
        assert_eq!(segments[1].text, "Second line");
    }

    #[test]
    fn parse_srt_segments_to_unified_model() {
        let srt = "1\n00:00:00,000 --> 00:00:01,200\nHello world\n\n2\n00:00:01,200 --> 00:00:02,500\nSecond line\n";

        let segments = parse_subtitle_content(srt, "srt").expect("parse srt");
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].text, "Hello world");
        assert_eq!(segments[1].end_ms, 2500);
    }

    #[test]
    fn render_srt_uses_standard_format() {
        let segments = vec![
            SubtitleSegment {
                index: 1,
                start_ms: 0,
                end_ms: 1200,
                text: "Hello world".to_string(),
            },
            SubtitleSegment {
                index: 2,
                start_ms: 1200,
                end_ms: 2500,
                text: "Second line".to_string(),
            },
        ];

        let rendered = render_srt(&segments);
        assert!(rendered.contains("1\n00:00:00,000 --> 00:00:01,200\nHello world"));
        assert!(rendered.contains("2\n00:00:01,200 --> 00:00:02,500\nSecond line"));
    }
}
