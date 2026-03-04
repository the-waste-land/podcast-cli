pub mod categories;
pub mod config;
pub mod download;
pub mod episodes;
pub mod recent;
pub mod search;
pub mod show;
pub mod stats;
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
        Commands::Download(args) => download::run(args, manager).await,
        Commands::Trending(args) => trending::run(args, manager).await,
        Commands::Recent(args) => recent::run(args, manager).await,
        Commands::Categories(args) => categories::run(args, manager).await,
        Commands::Stats(args) => stats::run(args, manager).await,
        Commands::Config(args) => config::run(args, manager),
    }
}
