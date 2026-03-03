use crate::api::client::PodcastIndexClient;
use crate::api::endpoints::categories::get_categories;
use crate::cli::CategoriesArgs;
use crate::config::ConfigManager;
use crate::error::Result;
use crate::output::json::to_pretty_json;
use crate::output::table::render_categories_list;
use crate::output::OutputFormat;

pub async fn run(args: CategoriesArgs, manager: &ConfigManager) -> Result<()> {
    let cfg = manager.load()?;
    let (api_key, api_secret) = cfg.require_credentials()?;
    let output = args.output.map(Into::into).unwrap_or(cfg.default_output);

    let client = PodcastIndexClient::new(api_key, api_secret);
    let response = get_categories(&client).await?;

    match output {
        OutputFormat::Json => println!("{}", to_pretty_json(&response)?),
        OutputFormat::Table => println!("{}", render_categories_list(&response.categories)),
    }

    Ok(())
}
