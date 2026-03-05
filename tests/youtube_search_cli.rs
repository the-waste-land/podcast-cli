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
            assert!(!args.with_meta);
            assert_eq!(args.meta_concurrency, None);
            assert_eq!(args.meta_timeout, None);
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
            assert!(!args.with_meta);
            assert_eq!(args.meta_concurrency, None);
            assert_eq!(args.meta_timeout, None);
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

#[test]
fn parse_youtube_search_with_meta_options() {
    let cli = Cli::parse_from([
        "podcast",
        "youtube-search",
        "rust",
        "--with-meta",
        "--meta-concurrency",
        "4",
        "--meta-timeout",
        "20",
    ]);

    match cli.command {
        Commands::YoutubeSearch(args) => {
            assert!(args.with_meta);
            assert_eq!(args.meta_concurrency, Some(4));
            assert_eq!(args.meta_timeout, Some(20));
        }
        _ => panic!("expected youtube-search command"),
    }
}

#[test]
fn reject_invalid_meta_concurrency() {
    let err = Cli::try_parse_from([
        "podcast",
        "youtube-search",
        "rust",
        "--meta-concurrency",
        "0",
    ])
    .expect_err("invalid --meta-concurrency should fail");

    assert!(err
        .to_string()
        .contains("meta-concurrency must be in range 1..=16"));
}

#[test]
fn reject_invalid_meta_timeout() {
    let err = Cli::try_parse_from(["podcast", "youtube-search", "rust", "--meta-timeout", "121"])
        .expect_err("invalid --meta-timeout should fail");

    assert!(err
        .to_string()
        .contains("meta-timeout must be in range 1..=120"));
}
