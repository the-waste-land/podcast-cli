use clap::Parser;
use podcast_cli::api::client::PodcastIndexClient;
use podcast_cli::api::endpoints::recent::get_recent_episodes;
use podcast_cli::api::types::{RecentEpisodesResponse, RecentFeedsResponse};
use podcast_cli::cli::{Cli, Commands, OutputArg};
use podcast_cli::commands::recent;
use podcast_cli::config::ConfigManager;
use podcast_cli::output::json::to_pretty_json;
use tempfile::tempdir;

#[test]
fn parse_recent_defaults_to_episodes_mode() {
    let cli = Cli::parse_from(["podcast", "recent", "--limit", "10"]);

    match cli.command {
        Commands::Recent(args) => {
            assert!(!args.feeds);
            assert_eq!(args.limit, Some(10));
            assert_eq!(args.before, None);
            assert_eq!(args.since, None);
            assert_eq!(args.output, None);
        }
        _ => panic!("expected recent command"),
    }
}

#[test]
fn parse_recent_feeds_mode_with_since_and_output() {
    let cli = Cli::parse_from([
        "podcast",
        "recent",
        "--feeds",
        "--since",
        "1700000000",
        "--output",
        "json",
    ]);

    match cli.command {
        Commands::Recent(args) => {
            assert!(args.feeds);
            assert_eq!(args.since, Some(1_700_000_000));
            assert_eq!(args.output, Some(OutputArg::Json));
        }
        _ => panic!("expected recent command"),
    }
}

#[test]
fn recent_rejects_non_numeric_before() {
    let err = Cli::try_parse_from(["podcast", "recent", "--before", "not-a-number"])
        .expect_err("expected invalid --before value");
    assert!(err
        .to_string()
        .contains("before must be an integer timestamp"));
}

#[test]
fn recent_rejects_non_numeric_since() {
    let err = Cli::try_parse_from(["podcast", "recent", "--feeds", "--since", "not-a-number"])
        .expect_err("expected invalid --since value");
    assert!(err
        .to_string()
        .contains("since must be an integer timestamp"));
}

#[test]
fn recent_episodes_json_output_contains_expected_keys() {
    let response = RecentEpisodesResponse {
        status: "true".to_string(),
        description: "ok".to_string(),
        count: 1,
        items: vec![],
    };

    let json = to_pretty_json(&response).expect("json serialize");
    assert!(json.contains("\"items\""));
    assert!(json.contains("\"count\""));
}

#[test]
fn recent_feeds_json_output_contains_expected_keys() {
    let response = RecentFeedsResponse {
        status: "true".to_string(),
        description: "ok".to_string(),
        count: 0,
        feeds: vec![],
    };

    let json = to_pretty_json(&response).expect("json serialize");
    assert!(json.contains("\"feeds\""));
    assert!(json.contains("\"count\""));
}

#[tokio::test]
async fn recent_command_fails_without_credentials() {
    let temp = tempdir().expect("create temp dir");
    let manager = ConfigManager::with_path(temp.path().join("podcast-cli.toml"));
    let args = podcast_cli::cli::RecentArgs {
        feeds: false,
        before: None,
        since: None,
        limit: Some(5),
        output: Some(OutputArg::Json),
    };

    let err = recent::run(args, &manager)
        .await
        .expect_err("expected missing config error");
    assert!(err.to_string().contains("api_key is not configured"));
}

#[tokio::test]
async fn recent_endpoint_surfaces_network_error() {
    let client = PodcastIndexClient::with_base_url("key", "secret", "http://127.0.0.1:1");
    let err = get_recent_episodes(&client, Some(1), None)
        .await
        .expect_err("expected connection failure");
    assert!(err.to_string().contains("HTTP error"));
}
