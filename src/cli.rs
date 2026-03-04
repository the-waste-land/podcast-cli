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
    #[arg(long, value_name = "path")]
    pub dest: Option<PathBuf>,
    #[arg(long, value_name = "name")]
    pub filename: Option<String>,
    #[arg(long, conflicts_with = "dry_run")]
    pub overwrite: bool,
    #[arg(long, conflicts_with = "dry_run")]
    pub resume: bool,
    #[arg(
        long,
        value_name = "seconds",
        default_value_t = 120,
        value_parser = parse_timeout
    )]
    pub timeout: u64,
    #[arg(long)]
    pub no_progress: bool,
    #[arg(long, conflicts_with = "no_progress")]
    pub progress_json: bool,
    #[arg(long, conflicts_with_all = ["resume", "overwrite"])]
    pub dry_run: bool,
    #[arg(
        long = "path-only",
        alias = "quiet",
        conflicts_with_all = ["minimal", "output"]
    )]
    pub path_only: bool,
    #[arg(long, conflicts_with_all = ["path_only", "output"])]
    pub minimal: bool,
    #[arg(long, value_enum, conflicts_with_all = ["path_only", "minimal"])]
    pub output: Option<OutputArg>,
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
