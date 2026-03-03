use clap::Parser;
use podcast_cli::cli::{Cli, Commands, OutputArg};

#[test]
fn parse_search_with_flags() {
    let cli = Cli::parse_from([
        "podcast", "search", "rust", "--limit", "5", "--output", "table", "--person",
    ]);

    match cli.command {
        Commands::Search(args) => {
            assert_eq!(args.term, "rust");
            assert_eq!(args.limit, Some(5));
            assert_eq!(args.output, Some(OutputArg::Table));
            assert!(args.person);
        }
        _ => panic!("expected search command"),
    }
}

#[test]
fn parse_show_by_id() {
    let cli = Cli::parse_from(["podcast", "show", "920666"]);

    match cli.command {
        Commands::Show(args) => assert_eq!(args.feed_id, Some(920666)),
        _ => panic!("expected show command"),
    }
}

#[test]
fn show_requires_id_or_url() {
    let result = Cli::try_parse_from(["podcast", "show"]);
    assert!(result.is_err());
}

#[test]
fn show_rejects_id_with_url() {
    let result = Cli::try_parse_from([
        "podcast",
        "show",
        "920666",
        "--url",
        "https://example.com/feed.xml",
    ]);
    assert!(result.is_err());
}
