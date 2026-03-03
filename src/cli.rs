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
