use podcast_cli::config::{AppConfig, ConfigManager};
use podcast_cli::output::OutputFormat;
use tempfile::tempdir;

#[test]
fn config_can_roundtrip_and_clear() {
    let temp = tempdir().expect("create temp dir");
    let path = temp.path().join("podcast-cli.toml");
    let manager = ConfigManager::with_path(path.clone());

    let cfg = AppConfig {
        api_key: Some("test_key".to_string()),
        api_secret: Some("test_secret".to_string()),
        default_output: OutputFormat::Json,
        max_results: 25,
    };

    manager.save(&cfg).expect("save should succeed");

    let loaded = manager.load().expect("load should succeed");
    assert_eq!(loaded.api_key.as_deref(), Some("test_key"));
    assert_eq!(loaded.api_secret.as_deref(), Some("test_secret"));
    assert_eq!(loaded.default_output, OutputFormat::Json);
    assert_eq!(loaded.max_results, 25);

    manager.clear().expect("clear should succeed");
    assert!(!path.exists());
}

#[test]
fn config_masks_sensitive_fields() {
    let cfg = AppConfig {
        api_key: Some("abcd1234".to_string()),
        api_secret: Some("secret9876".to_string()),
        ..AppConfig::default()
    };

    let masked = cfg.masked();
    assert_eq!(masked.api_key.as_deref(), Some("****1234"));
    assert_eq!(masked.api_secret.as_deref(), Some("******9876"));
}

#[test]
fn config_validation_fails_when_missing_credentials() {
    let cfg = AppConfig::default();
    let result = cfg.require_credentials();
    assert!(result.is_err());
}
