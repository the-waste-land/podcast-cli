use clap::Parser;
use podcast_cli::api::types::{Podcast, PodcastResponse, TrendingEpisodesResponse};
use podcast_cli::cli::{Cli, Commands, OutputArg};
use podcast_cli::output::json::to_pretty_json;

#[test]
fn parse_trending_defaults_to_podcast_mode() {
    let cli = Cli::parse_from(["podcast", "trending", "--limit", "12"]);

    match cli.command {
        Commands::Trending(args) => {
            assert_eq!(args.limit, Some(12));
            assert!(!args.episodes);
            assert_eq!(args.lang, None);
            assert_eq!(args.output, None);
        }
        _ => panic!("expected trending command"),
    }
}

#[test]
fn parse_trending_with_episode_mode() {
    let cli = Cli::parse_from([
        "podcast",
        "trending",
        "--episodes",
        "--lang",
        "en",
        "--limit",
        "8",
        "--output",
        "json",
    ]);

    match cli.command {
        Commands::Trending(args) => {
            assert!(args.episodes);
            assert_eq!(args.lang.as_deref(), Some("en"));
            assert_eq!(args.limit, Some(8));
            assert_eq!(args.output, Some(OutputArg::Json));
        }
        _ => panic!("expected trending command"),
    }
}

#[test]
fn trending_podcast_json_output_contains_expected_keys() {
    let response = PodcastResponse {
        status: "true".to_string(),
        description: "ok".to_string(),
        count: 1,
        feed: None,
        feeds: vec![Podcast {
            id: 920666,
            title: "Podcast A".to_string(),
            ..Podcast::default()
        }],
    };

    let json = to_pretty_json(&response).expect("json serialize");
    assert!(json.contains("\"feeds\""));
    assert!(json.contains("\"id\""));
}

#[test]
fn trending_episode_json_output_contains_expected_keys() {
    let response = TrendingEpisodesResponse {
        status: "true".to_string(),
        description: "ok".to_string(),
        count: 0,
        items: vec![],
    };

    let json = to_pretty_json(&response).expect("json serialize");
    assert!(json.contains("\"items\""));
    assert!(json.contains("\"count\""));
}
