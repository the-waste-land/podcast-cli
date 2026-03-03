use crate::api::client::PodcastIndexClient;
use crate::api::endpoints::episodes::get_trending_episodes;
use crate::api::endpoints::podcasts::get_trending_podcasts;
use crate::cli::TrendingArgs;
use crate::config::ConfigManager;
use crate::error::{PodcastCliError, Result};
use crate::output::json::to_pretty_json;
use crate::output::table::{render_episode_list, render_podcast_list};
use crate::output::OutputFormat;

pub async fn run(args: TrendingArgs, manager: &ConfigManager) -> Result<()> {
    let cfg = manager.load()?;
    let (api_key, api_secret) = cfg.require_credentials()?;

    let limit = args.limit.unwrap_or(cfg.max_results);
    validate_limit(limit)?;

    let output = args.output.map(Into::into).unwrap_or(cfg.default_output);
    let client = PodcastIndexClient::new(api_key, api_secret);

    if args.episodes {
        let response = get_trending_episodes(&client, limit, args.lang.as_deref()).await?;
        match output {
            OutputFormat::Json => println!("{}", to_pretty_json(&response)?),
            OutputFormat::Table => println!("{}", render_episode_list(&response.items)),
        }
    } else {
        let response = get_trending_podcasts(&client, limit, args.lang.as_deref()).await?;
        match output {
            OutputFormat::Json => println!("{}", to_pretty_json(&response)?),
            OutputFormat::Table => println!("{}", render_podcast_list(&response.feeds)),
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
