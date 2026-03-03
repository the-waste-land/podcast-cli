use crate::api::client::PodcastIndexClient;
use crate::api::endpoints::search::{search_by_person, search_by_term};
use crate::cli::SearchArgs;
use crate::config::ConfigManager;
use crate::error::{PodcastCliError, Result};
use crate::output::json::to_pretty_json;
use crate::output::table::render_podcast_list;
use crate::output::OutputFormat;

pub async fn run(args: SearchArgs, manager: &ConfigManager) -> Result<()> {
    let cfg = manager.load()?;
    let (api_key, api_secret) = cfg.require_credentials()?;

    let limit = args.limit.unwrap_or(cfg.max_results);
    validate_limit(limit)?;

    let output = args.output.map(Into::into).unwrap_or(cfg.default_output);

    let client = PodcastIndexClient::new(api_key, api_secret);
    let response = if args.person {
        search_by_person(&client, &args.term, limit, args.music).await?
    } else {
        search_by_term(&client, &args.term, limit, args.music).await?
    };

    match output {
        OutputFormat::Json => println!("{}", to_pretty_json(&response)?),
        OutputFormat::Table => println!("{}", render_podcast_list(&response.feeds)),
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
