use clap::Parser;
use podcast_cli::api::client::PodcastIndexClient;
use podcast_cli::api::endpoints::categories::get_categories;
use podcast_cli::api::types::{CategoriesResponse, Category};
use podcast_cli::cli::{Cli, Commands, OutputArg};
use podcast_cli::commands::categories;
use podcast_cli::config::ConfigManager;
use podcast_cli::output::json::to_pretty_json;
use tempfile::tempdir;

#[test]
fn parse_categories_defaults() {
    let cli = Cli::parse_from(["podcast", "categories"]);

    match cli.command {
        Commands::Categories(args) => {
            assert_eq!(args.output, None);
        }
        _ => panic!("expected categories command"),
    }
}

#[test]
fn parse_categories_with_json_output() {
    let cli = Cli::parse_from(["podcast", "categories", "--output", "json"]);

    match cli.command {
        Commands::Categories(args) => {
            assert_eq!(args.output, Some(OutputArg::Json));
        }
        _ => panic!("expected categories command"),
    }
}

#[test]
fn categories_json_output_contains_expected_keys() {
    let response = CategoriesResponse {
        status: "true".to_string(),
        description: "ok".to_string(),
        count: 1,
        feed_count: Some(100),
        categories: vec![Category {
            id: Some(11),
            name: Some("News".to_string()),
        }],
    };

    let json = to_pretty_json(&response).expect("json serialize");
    assert!(json.contains("\"categories\""));
    assert!(json.contains("\"name\""));
}

#[tokio::test]
async fn categories_command_fails_without_credentials() {
    let temp = tempdir().expect("create temp dir");
    let manager = ConfigManager::with_path(temp.path().join("podcast-cli.toml"));
    let args = podcast_cli::cli::CategoriesArgs {
        output: Some(OutputArg::Table),
    };

    let err = categories::run(args, &manager)
        .await
        .expect_err("expected missing config error");
    assert!(err.to_string().contains("api_key is not configured"));
}

#[tokio::test]
async fn categories_endpoint_surfaces_network_error() {
    let client = PodcastIndexClient::with_base_url("key", "secret", "http://127.0.0.1:1");
    let err = get_categories(&client)
        .await
        .expect_err("expected connection failure");
    assert!(err.to_string().contains("HTTP error"));
}
