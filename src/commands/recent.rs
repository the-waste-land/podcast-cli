use crate::api::client::PodcastIndexClient;
use crate::api::endpoints::recent::{get_recent_episodes, get_recent_feeds};
use crate::cli::RecentArgs;
use crate::config::ConfigManager;
use crate::error::{PodcastCliError, Result};
use crate::output::json::to_pretty_json;
use crate::output::table::{render_episode_list, render_podcast_list};
use crate::output::OutputFormat;

pub async fn run(args: RecentArgs, manager: &ConfigManager) -> Result<()> {
    let cfg = manager.load()?;
    let (api_key, api_secret) = cfg.require_credentials()?;

    let limit = args.limit.unwrap_or(cfg.max_results);
    validate_limit(limit)?;

    let output = args.output.map(Into::into).unwrap_or(cfg.default_output);
    let client = PodcastIndexClient::new(api_key, api_secret);

    if args.feeds {
        let response = get_recent_feeds(&client, Some(limit), args.since).await?;
        match output {
            OutputFormat::Json => println!("{}", to_pretty_json(&response)?),
            OutputFormat::Table => println!("{}", render_podcast_list(&response.feeds)),
        }
    } else {
        let response = get_recent_episodes(&client, Some(limit), args.before).await?;
        match output {
            OutputFormat::Json => println!("{}", to_pretty_json(&response)?),
            OutputFormat::Table => println!("{}", render_episode_list(&response.items)),
        }
    }

    Ok(())
}

fn validate_limit(limit: u32) -> Result<()> {
    if (1..=100).contains(&limit) {
        Ok(())
    } else {
        Err(PodcastCliError::Validation(
            "limit must be in range 1..=100".to_string(),
        ))
    }
}
