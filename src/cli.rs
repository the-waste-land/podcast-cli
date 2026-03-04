use std::path::PathBuf;

use clap::{ArgGroup, Args, Parser, Subcommand, ValueEnum};

use crate::output::OutputFormat;

#[derive(Debug, Parser)]
#[command(name = "podcast", version, about = "Podcast Index CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Search(SearchArgs),
    Show(ShowArgs),
    Episodes(EpisodesArgs),
    Episode(EpisodeArgs),
    Download(DownloadArgs),
    Transcribe(TranscribeArgs),
    YoutubeSubtitles(YoutubeSubtitlesArgs),
    YoutubeSearch(YoutubeSearchArgs),
    Trending(TrendingArgs),
    Recent(RecentArgs),
    Categories(CategoriesArgs),
    Stats(StatsArgs),
    Config(ConfigArgs),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lower")]
pub enum OutputArg {
    Json,
    Table,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lower")]
pub enum SubtitleOutputArg {
    Json,
    Text,
    Srt,
}

impl From<OutputArg> for OutputFormat {
    fn from(value: OutputArg) -> Self {
        match value {
            OutputArg::Json => OutputFormat::Json,
            OutputArg::Table => OutputFormat::Table,
        }
    }
}

#[derive(Debug, Args)]
pub struct SearchArgs {
    #[arg(value_name = "term")]
    pub term: String,
    #[arg(long, help = "Use /search/byperson endpoint")]
    pub person: bool,
    #[arg(long, help = "Limit results to music category")]
    pub music: bool,
    #[arg(long, value_name = "n")]
    pub limit: Option<u32>,
    #[arg(long, value_enum)]
    pub output: Option<OutputArg>,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("show_target")
        .required(true)
        .args(["feed_id", "url"])
))]
pub struct ShowArgs {
    #[arg(value_name = "feed-id", conflicts_with = "url")]
    pub feed_id: Option<u64>,
    #[arg(long, value_name = "feed-url", conflicts_with = "feed_id")]
    pub url: Option<String>,
    #[arg(long, value_enum)]
    pub output: Option<OutputArg>,
}

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigSubcommand,
}

#[derive(Debug, Args)]
pub struct EpisodesArgs {
    #[arg(value_name = "feed-id", value_parser = parse_feed_id)]
    pub feed_id: u64,
    #[arg(long, value_name = "n")]
    pub limit: Option<u32>,
    #[arg(long, value_enum)]
    pub output: Option<OutputArg>,
}

#[derive(Debug, Args)]
pub struct EpisodeArgs {
    #[arg(value_name = "episode-id", value_parser = parse_episode_id)]
    pub episode_id: u64,
    #[arg(long, value_enum)]
    pub output: Option<OutputArg>,
}

#[derive(Debug, Args)]
pub struct DownloadArgs {
    #[arg(value_name = "episode-id", value_parser = parse_episode_id)]
    pub episode_id: u64,
    #[arg(long, value_name = "path", help = "Download destination file or directory")]
    pub dest: Option<PathBuf>,
    #[arg(long, value_name = "name", help = "Override output filename")]
    pub filename: Option<String>,
    #[arg(long, conflicts_with = "dry_run", help = "Replace existing target file")]
    pub overwrite: bool,
    #[arg(long, conflicts_with = "dry_run", help = "Resume from existing .part file")]
    pub resume: bool,
    #[arg(
        long,
        value_name = "seconds",
        help = "HTTP timeout in seconds",
        default_value_t = 120,
        value_parser = parse_timeout
    )]
    pub timeout: u64,
    #[arg(long, help = "Disable progress output")]
    pub no_progress: bool,
    #[arg(long, conflicts_with = "no_progress", help = "Emit progress as JSON lines to stderr")]
    pub progress_json: bool,
    #[arg(
        long,
        conflicts_with_all = ["resume", "overwrite"],
        help = "Show resolved download metadata without downloading"
    )]
    pub dry_run: bool,
    #[arg(
        long = "path-only",
        alias = "quiet",
        help = "Print only the resolved output path",
        conflicts_with_all = ["minimal", "output"]
    )]
    pub path_only: bool,
    #[arg(
        long,
        help = "Emit compact JSON output for scripting",
        conflicts_with_all = ["path_only", "output"]
    )]
    pub minimal: bool,
    #[arg(
        long,
        value_enum,
        help = "Output format for full command results",
        conflicts_with_all = ["path_only", "minimal"]
    )]
    pub output: Option<OutputArg>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lower")]
pub enum TranscribeFormat {
    Json,
    Text,
    Srt,
}

#[derive(Debug, Args)]
#[command(group(
    ArgGroup::new("transcribe_input")
        .required(true)
        .args(["audio_file", "episode_id"])
))]
pub struct TranscribeArgs {
    #[arg(value_name = "audio-file", conflicts_with = "episode_id")]
    pub audio_file: Option<PathBuf>,
    #[arg(long, value_name = "episode-id", value_parser = parse_episode_id, conflicts_with = "audio_file")]
    pub episode_id: Option<u64>,
    #[arg(long, help = "Whisper model to use", default_value = "base")]
    pub model: String,
    #[arg(long, value_name = "code", help = "Language code", default_value = "en")]
    pub language: String,
    #[arg(long, value_enum, default_value_t = TranscribeFormat::Text)]
    pub format: TranscribeFormat,
    #[arg(long, value_name = "path", help = "Output file path")]
    pub output: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct YoutubeSubtitlesArgs {
    #[arg(value_name = "video-id", value_parser = parse_youtube_video_id)]
    pub video_id: String,
    #[arg(long, value_name = "code", default_value = "en", value_parser = parse_lang_code)]
    pub lang: String,
    #[arg(long, value_enum, default_value_t = SubtitleOutputArg::Json)]
    pub output: SubtitleOutputArg,
}

#[derive(Debug, Args)]
pub struct YoutubeSearchArgs {
    #[arg(value_name = "query", value_parser = parse_non_empty)]
    pub query: String,
    #[arg(long, value_name = "n")]
    pub limit: Option<u32>,
    #[arg(long, value_name = "name", help = "Filter results by channel name", value_parser = parse_non_empty)]
    pub channel: Option<String>,
    #[arg(long, value_name = "range", help = "Only include videos uploaded within range, e.g. 7d, 30d", value_parser = parse_youtube_since)]
    pub since: Option<String>,
}

#[derive(Debug, Args)]
pub struct TrendingArgs {
    #[arg(long, help = "Use /episodes/trending endpoint")]
    pub episodes: bool,
    #[arg(long, value_name = "code")]
    pub lang: Option<String>,
    #[arg(long, value_name = "n")]
    pub limit: Option<u32>,
    #[arg(long, value_enum)]
    pub output: Option<OutputArg>,
}

#[derive(Debug, Args)]
pub struct RecentArgs {
    #[arg(long, help = "Use /recent/feeds endpoint")]
    pub feeds: bool,
    #[arg(
        long,
        value_name = "unix-timestamp",
        value_parser = parse_before,
        conflicts_with = "feeds"
    )]
    pub before: Option<i64>,
    #[arg(
        long,
        value_name = "unix-timestamp",
        value_parser = parse_since,
        requires = "feeds"
    )]
    pub since: Option<i64>,
    #[arg(long, value_name = "n")]
    pub limit: Option<u32>,
    #[arg(long, value_enum)]
    pub output: Option<OutputArg>,
}

#[derive(Debug, Args)]
pub struct CategoriesArgs {
    #[arg(long, value_enum)]
    pub output: Option<OutputArg>,
}

#[derive(Debug, Args)]
pub struct StatsArgs {
    #[arg(long, value_enum)]
    pub output: Option<OutputArg>,
}

#[derive(Debug, Subcommand)]
pub enum ConfigSubcommand {
    Set(ConfigSetArgs),
    Show,
    Clear,
}

#[derive(Debug, Args)]
pub struct ConfigSetArgs {
    #[arg(long)]
    pub api_key: String,
    #[arg(long)]
    pub api_secret: String,
    #[arg(long, value_enum)]
    pub default_output: Option<OutputArg>,
    #[arg(long)]
    pub max_results: Option<u32>,
}

fn parse_feed_id(value: &str) -> std::result::Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|_| "feed-id must be an integer".to_string())
}

fn parse_episode_id(value: &str) -> std::result::Result<u64, String> {
    value
        .parse::<u64>()
        .map_err(|_| "episode-id must be an integer".to_string())
}

fn parse_before(value: &str) -> std::result::Result<i64, String> {
    value
        .parse::<i64>()
        .map_err(|_| "before must be an integer timestamp".to_string())
}

fn parse_since(value: &str) -> std::result::Result<i64, String> {
    value
        .parse::<i64>()
        .map_err(|_| "since must be an integer timestamp".to_string())
}

fn parse_timeout(value: &str) -> std::result::Result<u64, String> {
    let timeout = value
        .parse::<u64>()
        .map_err(|_| "timeout must be an integer".to_string())?;

    if timeout == 0 {
        return Err("timeout must be greater than 0".to_string());
    }

    Ok(timeout)
}

fn parse_lang_code(value: &str) -> std::result::Result<String, String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err("lang must not be empty".to_string());
    }

    Ok(normalized.to_string())
}

fn parse_youtube_since(value: &str) -> std::result::Result<String, String> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.len() < 2 {
        return Err("since must be like 7d, 2w, 1m, or 1y".to_string());
    }

    let split_at = normalized.len() - 1;
    let (amount_raw, unit) = normalized.split_at(split_at);
    let amount = amount_raw
        .parse::<u32>()
        .map_err(|_| "since must be like 7d, 2w, 1m, or 1y".to_string())?;

    if amount == 0 {
        return Err("since must be greater than 0".to_string());
    }

    let suffix = match unit {
        "d" => "days",
        "w" => "weeks",
        "m" => "months",
        "y" => "years",
        _ => return Err("since must be like 7d, 2w, 1m, or 1y".to_string()),
    };

    Ok(format!("now-{amount}{suffix}"))
}

fn parse_non_empty(value: &str) -> std::result::Result<String, String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err("value must not be empty".to_string());
    }

    Ok(normalized.to_string())
}

fn parse_youtube_video_id(value: &str) -> std::result::Result<String, String> {
    if value.len() != 11 {
        return Err("video-id must be 11 chars and use [A-Za-z0-9_-]".to_string());
    }

    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        Ok(value.to_string())
    } else {
        Err("video-id must be 11 chars and use [A-Za-z0-9_-]".to_string())
    }
}
