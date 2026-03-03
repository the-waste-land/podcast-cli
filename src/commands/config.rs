use crate::cli::{ConfigArgs, ConfigSubcommand};
use crate::config::ConfigManager;
use crate::error::{PodcastCliError, Result};
use crate::output::json::to_pretty_json;

pub fn run(args: ConfigArgs, manager: &ConfigManager) -> Result<()> {
    match args.command {
        ConfigSubcommand::Set(set) => {
            if let Some(max) = set.max_results {
                validate_limit(max)?;
            }

            let mut cfg = manager.load()?;
            cfg.api_key = Some(set.api_key);
            cfg.api_secret = Some(set.api_secret);
            if let Some(output) = set.default_output {
                cfg.default_output = output.into();
            }
            if let Some(max) = set.max_results {
                cfg.max_results = max;
            }

            manager.save(&cfg)?;
            println!("Configuration saved.");
            Ok(())
        }
        ConfigSubcommand::Show => {
            let cfg = manager.load()?.masked();
            println!("{}", to_pretty_json(&cfg)?);
            Ok(())
        }
        ConfigSubcommand::Clear => {
            manager.clear()?;
            println!("Configuration cleared.");
            Ok(())
        }
    }
}

fn validate_limit(limit: u32) -> Result<()> {
    if (1..=100).contains(&limit) {
        Ok(())
    } else {
        Err(PodcastCliError::Validation(
            "max_results must be in range 1..=100".to_string(),
        ))
    }
}
