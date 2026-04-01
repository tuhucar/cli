use clap::Subcommand;
use tuhucar_core::config::Config;
use tuhucar_core::{OutputFormat, TuhucarError};

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Initialize configuration
    Init,
    /// Show current configuration
    Show,
}

pub async fn run(action: ConfigAction, _format: OutputFormat) -> Result<(), TuhucarError> {
    match action {
        ConfigAction::Init => {
            let config = Config::default_config();
            config.save()?;
            println!("Configuration saved to {}", Config::config_path().display());
            println!("Edit {} to customize settings.", Config::config_path().display());
            Ok(())
        }
        ConfigAction::Show => {
            let config = Config::load()?;
            println!("{}", toml::to_string_pretty(&config).unwrap());
            Ok(())
        }
    }
}
