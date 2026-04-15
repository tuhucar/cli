pub mod config;
pub mod knowledge;
pub mod skill;

use clap::Subcommand;
use tuhucar_core::{OutputFormat, ResponseMeta, TuhucarError};

#[derive(Subcommand)]
pub enum Commands {
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
    meta: ResponseMeta,
) -> Result<(), TuhucarError> {
    match cmd {
        Commands::Knowledge { action } => knowledge::run(action, format, dry_run, verbose, meta).await,
        Commands::Config { action } => config::run(action, format, meta).await,
        Commands::Skill { action } => skill::run(action, format).await,
    }
}
