pub mod config;
pub mod search;
pub mod show;

use crate::cli::Commands;
use crate::config::ConfigManager;
use crate::error::Result;

pub async fn dispatch(command: Commands, manager: &ConfigManager) -> Result<()> {
    match command {
        Commands::Search(args) => search::run(args, manager).await,
        Commands::Show(args) => show::run(args, manager).await,
        Commands::Config(args) => config::run(args, manager),
    }
}
