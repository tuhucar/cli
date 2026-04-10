use clap::Subcommand;
use tuhucar_core::config::Config;
use tuhucar_core::{OutputFormat, Response, ResponseMeta, TuhucarError};

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Initialize configuration
    Init,
    /// Show current configuration
    Show,
}

pub async fn run(
    action: ConfigAction,
    format: OutputFormat,
    meta: ResponseMeta,
) -> Result<(), TuhucarError> {
    match action {
        ConfigAction::Init => {
            let config = Config::default_config();
            config.save()?;
            let path = Config::config_path().display().to_string();
            match format {
                OutputFormat::Json => {
                    let data = serde_json::json!({
                        "path": path,
                        "message": format!("Configuration saved to {}", path),
                    });
                    let resp = Response::success(data, Some(meta));
                    println!("{}", serde_json::to_string_pretty(&resp).unwrap());
                }
                OutputFormat::Markdown => {
                    println!("Configuration saved to {}", path);
                    println!("Edit {} to customize settings.", path);
                }
            }
            Ok(())
        }
        ConfigAction::Show => {
            let config = Config::load()?;
            match format {
                OutputFormat::Json => {
                    let data = serde_json::to_value(&config).unwrap();
                    let resp = Response::success(data, Some(meta));
                    println!("{}", serde_json::to_string_pretty(&resp).unwrap());
                }
                OutputFormat::Markdown => {
                    println!("{}", toml::to_string_pretty(&config).unwrap());
                }
            }
            Ok(())
        }
    }
}
