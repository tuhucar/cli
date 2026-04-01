pub mod car;
pub mod config;
pub mod knowledge;
pub mod skill;

use clap::Subcommand;
use tuhucar_core::{OutputFormat, TuhucarError};

#[derive(Subcommand)]
pub enum Commands {
    /// Car model operations
    Car {
        #[command(subcommand)]
        action: car::CarAction,
    },
    /// Knowledge query operations
    Knowledge {
        #[command(subcommand)]
        action: knowledge::KnowledgeAction,
    },
    /// Configuration management
    Config {
        #[command(subcommand)]
        action: config::ConfigAction,
    },
    /// Skill installation management
    Skill {
        #[command(subcommand)]
        action: skill::SkillAction,
    },
}

pub async fn run(
    cmd: Commands,
    format: OutputFormat,
    dry_run: bool,
    verbose: bool,
) -> Result<(), TuhucarError> {
    match cmd {
        Commands::Car { action } => car::run(action, format, dry_run, verbose).await,
        Commands::Knowledge { action } => knowledge::run(action, format, dry_run, verbose).await,
        Commands::Config { action } => config::run(action, format).await,
        Commands::Skill { action } => skill::run(action, format).await,
    }
}
