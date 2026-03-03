pub mod config;
pub mod episodes;
pub mod search;
pub mod show;
pub mod trending;

use crate::cli::Commands;
use crate::config::ConfigManager;
use crate::error::Result;

pub async fn dispatch(command: Commands, manager: &ConfigManager) -> Result<()> {
    match command {
        Commands::Search(args) => search::run(args, manager).await,
        Commands::Show(args) => show::run(args, manager).await,
        Commands::Episodes(args) => episodes::run_episodes(args, manager).await,
        Commands::Episode(args) => episodes::run_episode(args, manager).await,
        Commands::Trending(args) => trending::run(args, manager).await,
        Commands::Config(args) => config::run(args, manager),
    }
}
