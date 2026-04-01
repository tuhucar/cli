use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::TuhucarError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub api: ApiConfig,
    #[serde(default)]
    pub output: OutputConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiConfig {
    pub base_url: String,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutputConfig {
    #[serde(default = "default_format")]
    pub default_format: String,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            default_format: default_format(),
        }
    }
}

fn default_timeout() -> u64 { 30 }
fn default_format() -> String { "markdown".to_string() }

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .expect("Cannot determine home directory")
            .join(".tuhucar")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> Result<Self, TuhucarError> {
        let path = Self::config_path();
        if !path.exists() {
            return Err(TuhucarError::ConfigMissing {
                suggestion: "Run: tuhucar config init".into(),
            });
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| TuhucarError::ConfigParse(format!("{}: {}", path.display(), e)))?;
        toml::from_str(&content)
            .map_err(|e| TuhucarError::ConfigParse(format!("{}: {}", path.display(), e)))
    }

    pub fn save(&self) -> Result<(), TuhucarError> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)
            .map_err(|e| TuhucarError::ConfigParse(format!("Cannot create dir: {}", e)))?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| TuhucarError::ConfigParse(e.to_string()))?;
        std::fs::write(Self::config_path(), content)
            .map_err(|e| TuhucarError::ConfigParse(e.to_string()))
    }

    pub fn default_config() -> Self {
        Self {
            api: ApiConfig {
                base_url: "https://api.tuhucar.com".into(),
                timeout: 30,
            },
            output: OutputConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_config() {
        let toml_str = r#"
[api]
base_url = "https://api.example.com"
timeout = 15

[output]
default_format = "json"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.api.base_url, "https://api.example.com");
        assert_eq!(config.api.timeout, 15);
        assert_eq!(config.output.default_format, "json");
    }

    #[test]
    fn parse_minimal_config_uses_defaults() {
        let toml_str = r#"
[api]
base_url = "https://api.example.com"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.api.timeout, 30);
        assert_eq!(config.output.default_format, "markdown");
    }

    #[test]
    fn invalid_toml_returns_parse_error() {
        let toml_str = "not valid toml [[[";
        let result: Result<Config, _> = toml::from_str(toml_str);
        assert!(result.is_err());
    }

    #[test]
    fn default_config_has_expected_values() {
        let config = Config::default_config();
        assert_eq!(config.api.base_url, "https://api.tuhucar.com");
        assert_eq!(config.api.timeout, 30);
        assert_eq!(config.output.default_format, "markdown");
    }
}
