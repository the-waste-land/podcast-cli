use clap::Parser;
use podcast_cli::cli::{Cli, Commands, OutputArg};

#[test]
fn parse_download_with_required_episode_id() {
    let cli = Cli::parse_from(["podcast", "download", "123456"]);

    match cli.command {
        Commands::Download(args) => {
            assert_eq!(args.episode_id, 123456);
            assert!(args.dest.is_none());
            assert!(args.filename.is_none());
            assert!(!args.dry_run);
        }
        _ => panic!("expected download command"),
    }
}

#[test]
fn parse_download_with_path_only_alias_quiet() {
    let cli = Cli::parse_from(["podcast", "download", "123456", "--quiet"]);

    match cli.command {
        Commands::Download(args) => {
            assert!(args.path_only);
        }
        _ => panic!("expected download command"),
    }
}

#[test]
fn parse_download_with_dry_run_and_minimal() {
    let cli = Cli::parse_from(["podcast", "download", "123456", "--dry-run", "--minimal"]);

    match cli.command {
        Commands::Download(args) => {
            assert!(args.dry_run);
            assert!(args.minimal);
        }
        _ => panic!("expected download command"),
    }
}

#[test]
fn reject_path_only_with_output() {
    let err = Cli::try_parse_from([
        "podcast",
        "download",
        "123456",
        "--path-only",
        "--output",
        "json",
    ])
    .expect_err("path-only should conflict with output");

    assert!(err.to_string().contains("--path-only"));
    assert!(err.to_string().contains("--output"));
}

#[test]
fn reject_minimal_with_output() {
    let err = Cli::try_parse_from([
        "podcast",
        "download",
        "123456",
        "--minimal",
        "--output",
        "table",
    ])
    .expect_err("minimal should conflict with output");

    assert!(err.to_string().contains("--minimal"));
    assert!(err.to_string().contains("--output"));
}

#[test]
fn reject_progress_json_with_no_progress() {
    let err = Cli::try_parse_from([
        "podcast",
        "download",
        "123456",
        "--progress-json",
        "--no-progress",
    ])
    .expect_err("progress-json should conflict with no-progress");

    assert!(err.to_string().contains("--progress-json"));
    assert!(err.to_string().contains("--no-progress"));
}

#[test]
fn reject_dry_run_with_resume() {
    let err = Cli::try_parse_from(["podcast", "download", "123456", "--dry-run", "--resume"])
        .expect_err("dry-run should conflict with resume");

    assert!(err.to_string().contains("--dry-run"));
    assert!(err.to_string().contains("--resume"));
}

#[test]
fn reject_dry_run_with_overwrite() {
    let err = Cli::try_parse_from(["podcast", "download", "123456", "--dry-run", "--overwrite"])
        .expect_err("dry-run should conflict with overwrite");

    assert!(err.to_string().contains("--dry-run"));
    assert!(err.to_string().contains("--overwrite"));
}

#[test]
fn download_rejects_non_numeric_episode_id() {
    let err = Cli::try_parse_from(["podcast", "download", "not-a-number"])
        .expect_err("expected invalid episode-id to fail");

    assert!(err.to_string().contains("episode-id must be an integer"));
}

#[test]
fn download_rejects_zero_timeout() {
    let err = Cli::try_parse_from(["podcast", "download", "123456", "--timeout", "0"])
        .expect_err("timeout=0 should fail");

    assert!(err.to_string().contains("timeout must be greater than 0"));
}

#[test]
fn parse_download_with_output() {
    let cli = Cli::parse_from(["podcast", "download", "123456", "--output", "json"]);

    match cli.command {
        Commands::Download(args) => {
            assert_eq!(args.output, Some(OutputArg::Json));
        }
        _ => panic!("expected download command"),
    }
}
