use crate::api::client::PodcastIndexClient;
use crate::api::endpoints::podcasts::{podcast_by_feed_id, podcast_by_feed_url};
use crate::cli::ShowArgs;
use crate::config::ConfigManager;
use crate::error::{PodcastCliError, Result};
use crate::output::json::to_pretty_json;
use crate::output::table::render_podcast_detail;
use crate::output::OutputFormat;

pub async fn run(args: ShowArgs, manager: &ConfigManager) -> Result<()> {
    let cfg = manager.load()?;
    let (api_key, api_secret) = cfg.require_credentials()?;
    let output = args.output.map(Into::into).unwrap_or(cfg.default_output);

    let client = PodcastIndexClient::new(api_key, api_secret);
    let response = if let Some(feed_id) = args.feed_id {
        podcast_by_feed_id(&client, feed_id).await?
    } else if let Some(feed_url) = args.url.as_deref() {
        podcast_by_feed_url(&client, feed_url).await?
    } else {
        return Err(PodcastCliError::Validation(
            "either feed-id or --url must be provided".to_string(),
        ));
    };

    match output {
        OutputFormat::Json => println!("{}", to_pretty_json(&response)?),
        OutputFormat::Table => {
            if let Some(podcast) = response.first_podcast() {
                println!("{}", render_podcast_detail(podcast));
            } else {
                println!("Podcast not found.");
            }
        }
    }

    Ok(())
}
