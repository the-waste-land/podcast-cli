use clap::Parser;
use podcast_cli::cli::{Cli, Commands};

#[test]
fn parse_youtube_search_defaults() {
    let cli = Cli::parse_from(["podcast", "youtube-search", "rust podcast"]);

    match cli.command {
        Commands::YoutubeSearch(args) => {
            assert_eq!(args.query, "rust podcast");
            assert_eq!(args.limit, None);
            assert_eq!(args.channel, None);
            assert_eq!(args.since, None);
        }
        _ => panic!("expected youtube-search command"),
    }
}

#[test]
fn parse_youtube_search_with_filters() {
    let cli = Cli::parse_from([
        "podcast",
        "youtube-search",
        "rust",
        "--limit",
        "5",
        "--channel",
        "ThePrimeTime",
        "--since",
        "30d",
    ]);

    match cli.command {
        Commands::YoutubeSearch(args) => {
            assert_eq!(args.query, "rust");
            assert_eq!(args.limit, Some(5));
            assert_eq!(args.channel.as_deref(), Some("ThePrimeTime"));
            assert_eq!(args.since.as_deref(), Some("now-30days"));
        }
        _ => panic!("expected youtube-search command"),
    }
}

#[test]
fn reject_invalid_since_format() {
    let err = Cli::try_parse_from(["podcast", "youtube-search", "rust", "--since", "abc"])
        .expect_err("invalid --since should fail");

    assert!(err
        .to_string()
        .contains("since must be like 7d, 2w, 1m, or 1y"));
}
