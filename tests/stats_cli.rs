use clap::Parser;
use podcast_cli::api::client::PodcastIndexClient;
use podcast_cli::api::endpoints::stats::get_stats;
use podcast_cli::api::types::{Stats, StatsResponse};
use podcast_cli::cli::{Cli, Commands, OutputArg};
use podcast_cli::commands::stats;
use podcast_cli::config::ConfigManager;
use podcast_cli::output::json::to_pretty_json;
use tempfile::tempdir;

#[test]
fn parse_stats_defaults() {
    let cli = Cli::parse_from(["podcast", "stats"]);

    match cli.command {
        Commands::Stats(args) => {
            assert_eq!(args.output, None);
        }
        _ => panic!("expected stats command"),
    }
}

#[test]
fn parse_stats_with_table_output() {
    let cli = Cli::parse_from(["podcast", "stats", "--output", "table"]);

    match cli.command {
        Commands::Stats(args) => {
            assert_eq!(args.output, Some(OutputArg::Table));
        }
        _ => panic!("expected stats command"),
    }
}

#[test]
fn stats_json_output_contains_expected_keys() {
    let response = StatsResponse {
        status: "true".to_string(),
        description: "ok".to_string(),
        stats: Stats {
            feed_count_total: Some(2_000_000),
            episode_count_total: Some(100_000_000),
            feeds_with_new_episodes_3days: Some(20_000),
            feeds_with_new_episodes_10days: Some(60_000),
            feeds_with_new_episodes_30days: Some(120_000),
            feeds_with_value_blocks: Some(3_000),
        },
    };

    let json = to_pretty_json(&response).expect("json serialize");
    assert!(json.contains("\"stats\""));
    assert!(json.contains("\"feedCountTotal\""));
}

#[tokio::test]
async fn stats_command_fails_without_credentials() {
    let temp = tempdir().expect("create temp dir");
    let manager = ConfigManager::with_path(temp.path().join("podcast-cli.toml"));
    let args = podcast_cli::cli::StatsArgs {
        output: Some(OutputArg::Json),
    };

    let err = stats::run(args, &manager)
        .await
        .expect_err("expected missing config error");
    assert!(err.to_string().contains("api_key is not configured"));
}

#[tokio::test]
async fn stats_endpoint_surfaces_network_error() {
    let client = PodcastIndexClient::with_base_url("key", "secret", "http://127.0.0.1:1");
    let err = get_stats(&client)
        .await
        .expect_err("expected connection failure");
    assert!(err.to_string().contains("HTTP error"));
}
