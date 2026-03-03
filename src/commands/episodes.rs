use crate::api::client::PodcastIndexClient;
use crate::api::endpoints::episodes::{get_episode_by_id, get_episodes_by_feed_id};
use crate::cli::{EpisodeArgs, EpisodesArgs};
use crate::config::ConfigManager;
use crate::error::{PodcastCliError, Result};
use crate::output::json::to_pretty_json;
use crate::output::table::{render_episode_detail, render_episode_list};
use crate::output::OutputFormat;

pub async fn run_episodes(args: EpisodesArgs, manager: &ConfigManager) -> Result<()> {
    let cfg = manager.load()?;
    let (api_key, api_secret) = cfg.require_credentials()?;

    let limit = args.limit.unwrap_or(cfg.max_results);
    validate_limit(limit)?;

    let output = args.output.map(Into::into).unwrap_or(cfg.default_output);
    let client = PodcastIndexClient::new(api_key, api_secret);
    let response = get_episodes_by_feed_id(&client, args.feed_id, limit).await?;

    match output {
        OutputFormat::Json => println!("{}", to_pretty_json(&response)?),
        OutputFormat::Table => println!("{}", render_episode_list(&response.items)),
    }

    Ok(())
}

pub async fn run_episode(args: EpisodeArgs, manager: &ConfigManager) -> Result<()> {
    let cfg = manager.load()?;
    let (api_key, api_secret) = cfg.require_credentials()?;
    let output = args.output.map(Into::into).unwrap_or(cfg.default_output);

    let client = PodcastIndexClient::new(api_key, api_secret);
    let response = get_episode_by_id(&client, args.episode_id).await?;

    match output {
        OutputFormat::Json => println!("{}", to_pretty_json(&response)?),
        OutputFormat::Table => {
            if let Some(episode) = response.first_episode() {
                println!("{}", render_episode_detail(episode));
            } else {
                println!("Episode not found.");
            }
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
