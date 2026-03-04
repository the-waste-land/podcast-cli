use clap::Parser;
use podcast_cli::cli::{Cli, Commands, SubtitleOutputArg};

#[test]
fn parse_youtube_subtitles_defaults() {
    let cli = Cli::parse_from(["podcast", "youtube-subtitles", "dQw4w9WgXcQ"]);

    match cli.command {
        Commands::YoutubeSubtitles(args) => {
            assert_eq!(args.video_id, "dQw4w9WgXcQ");
            assert_eq!(args.lang, "en");
            assert_eq!(args.output, SubtitleOutputArg::Json);
        }
        _ => panic!("expected youtube-subtitles command"),
    }
}

#[test]
fn parse_youtube_subtitles_with_explicit_args() {
    let cli = Cli::parse_from([
        "podcast",
        "youtube-subtitles",
        "dQw4w9WgXcQ",
        "--lang",
        "zh",
        "--output",
        "srt",
    ]);

    match cli.command {
        Commands::YoutubeSubtitles(args) => {
            assert_eq!(args.video_id, "dQw4w9WgXcQ");
            assert_eq!(args.lang, "zh");
            assert_eq!(args.output, SubtitleOutputArg::Srt);
        }
        _ => panic!("expected youtube-subtitles command"),
    }
}

#[test]
fn reject_invalid_video_id() {
    let err = Cli::try_parse_from(["podcast", "youtube-subtitles", "bad-id"])
        .expect_err("invalid video-id should fail");

    assert!(err.to_string().contains("video-id must be 11 chars"));
}
