use clap::Parser;
use podcast_cli::cli::{Cli, Commands, OutputArg};

#[test]
fn parse_youtube_meta_defaults() {
    let cli = Cli::parse_from(["podcast", "youtube-meta", "dQw4w9WgXcQ"]);

    match cli.command {
        Commands::YoutubeMeta(args) => {
            assert_eq!(args.video_id, "dQw4w9WgXcQ");
            assert_eq!(args.output, None);
        }
        _ => panic!("expected youtube-meta command"),
    }
}

#[test]
fn parse_youtube_meta_with_output() {
    let cli = Cli::parse_from([
        "podcast",
        "youtube-meta",
        "dQw4w9WgXcQ",
        "--output",
        "table",
    ]);

    match cli.command {
        Commands::YoutubeMeta(args) => {
            assert_eq!(args.video_id, "dQw4w9WgXcQ");
            assert_eq!(args.output, Some(OutputArg::Table));
        }
        _ => panic!("expected youtube-meta command"),
    }
}

#[test]
fn reject_invalid_video_id_for_youtube_meta() {
    let err = Cli::try_parse_from(["podcast", "youtube-meta", "bad-id"])
        .expect_err("invalid video-id should fail");

    assert!(err.to_string().contains("video-id must be 11 chars"));
}
