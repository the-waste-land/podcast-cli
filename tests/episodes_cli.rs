use clap::Parser;
use podcast_cli::api::types::{Episode, EpisodeResponse, EpisodesResponse};
use podcast_cli::cli::{Cli, Commands, OutputArg};
use podcast_cli::output::json::to_pretty_json;

#[test]
fn parse_episodes_with_limit_and_output() {
    let cli = Cli::parse_from([
        "podcast", "episodes", "920666", "--limit", "10", "--output", "json",
    ]);

    match cli.command {
        Commands::Episodes(args) => {
            assert_eq!(args.feed_id, 920666);
            assert_eq!(args.limit, Some(10));
            assert_eq!(args.output, Some(OutputArg::Json));
        }
        _ => panic!("expected episodes command"),
    }
}

#[test]
fn parse_episode_with_output() {
    let cli = Cli::parse_from(["podcast", "episode", "123456", "--output", "table"]);

    match cli.command {
        Commands::Episode(args) => {
            assert_eq!(args.episode_id, 123456);
            assert_eq!(args.output, Some(OutputArg::Table));
        }
        _ => panic!("expected episode command"),
    }
}

#[test]
fn episodes_rejects_non_numeric_feed_id() {
    let err = Cli::try_parse_from(["podcast", "episodes", "not-a-number"])
        .expect_err("expected invalid feed-id to fail");
    assert!(err.to_string().contains("feed-id must be an integer"));
}

#[test]
fn episode_rejects_non_numeric_episode_id() {
    let err = Cli::try_parse_from(["podcast", "episode", "not-a-number"])
        .expect_err("expected invalid episode-id to fail");
    assert!(err.to_string().contains("episode-id must be an integer"));
}

#[test]
fn episodes_json_output_contains_expected_keys() {
    let response = EpisodesResponse {
        status: "true".to_string(),
        description: "ok".to_string(),
        count: 1,
        items: vec![Episode {
            id: Some(100001),
            title: Some("Episode Title".to_string()),
            date_published: Some(1_700_000_000),
            duration: Some(3600),
            feed_id: Some(920666),
            feed_title: Some("Test Feed".to_string()),
            ..Episode::default()
        }],
    };

    let json = to_pretty_json(&response).expect("json serialize");
    assert!(json.contains("\"items\""));
    assert!(json.contains("\"feedId\""));
}

#[test]
fn episode_json_output_contains_expected_keys() {
    let response = EpisodeResponse {
        status: "true".to_string(),
        description: "ok".to_string(),
        item: Some(Episode {
            id: Some(200001),
            title: Some("Single Episode".to_string()),
            ..Episode::default()
        }),
        items: vec![],
    };

    let json = to_pretty_json(&response).expect("json serialize");
    assert!(json.contains("\"item\""));
    assert!(json.contains("\"title\""));
}
