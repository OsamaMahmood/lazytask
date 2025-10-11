use anyhow::Result;
use clap::Parser;

mod app;
mod config;
mod taskwarrior;
mod ui;
mod handlers;
mod data;
mod utils;

use app::App;

#[derive(Parser)]
#[command(
    name = "lazytask",
    about = "A modern Terminal User Interface for Taskwarrior",
    version
)]
struct Cli {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    let mut app = App::new(cli.config.as_deref(), cli.verbose)?;
    app.run().await?;
    
    Ok(())
}
