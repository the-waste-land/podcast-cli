use clap::Parser;
use podcast_cli::cli::Cli;
use podcast_cli::commands;
use podcast_cli::config::ConfigManager;
use podcast_cli::error::Result;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let manager = ConfigManager::new();
    commands::dispatch(cli.command, &manager).await
}
