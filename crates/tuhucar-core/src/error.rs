use serde::Serialize;
use schemars::JsonSchema;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TuhucarError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Config missing: {suggestion}")]
    ConfigMissing { suggestion: String },
    #[error("Config parse error: {0}")]
    ConfigParse(String),
    #[error("Car not found")]
    CarNotFound { suggestion: String },
    #[error("MCP error ({code}): {message}")]
    McpError { code: i64, message: String },
    #[error("Invalid arguments: {message}")]
    InvalidArgs { message: String, suggestion: String },
}

#[derive(Debug, Serialize, JsonSchema, Clone)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream: Option<UpstreamError>,
}

#[derive(Debug, Serialize, JsonSchema, Clone)]
pub struct UpstreamError {
    pub status: u16,
    pub code: String,
    pub message: String,
}

impl From<TuhucarError> for ApiError {
    fn from(err: TuhucarError) -> Self {
        match err {
            TuhucarError::Network(_) => ApiError {
                code: "NETWORK_ERROR".into(),
                message: err.to_string(),
                retryable: true,
                suggestion: None,
                upstream: None,
            },
            TuhucarError::ConfigMissing { suggestion } => ApiError {
                code: "CONFIG_MISSING".into(),
                message: "Configuration not found".into(),
                retryable: false,
                suggestion: Some(suggestion),
                upstream: None,
            },
            TuhucarError::ConfigParse(msg) => ApiError {
                code: "CONFIG_PARSE_ERROR".into(),
                message: msg,
                retryable: false,
                suggestion: Some("Check ~/.tuhucar/config.toml for syntax errors".into()),
                upstream: None,
            },
            TuhucarError::CarNotFound { suggestion } => ApiError {
                code: "CAR_NOT_FOUND".into(),
                message: "No matching car model found".into(),
                retryable: false,
                suggestion: Some(suggestion),
                upstream: None,
            },
            TuhucarError::McpError { code, message } => ApiError {
                code: "MCP_ERROR".into(),
                message: message.clone(),
                retryable: code >= 500,
                suggestion: None,
                upstream: Some(UpstreamError {
                    status: code as u16,
                    code: "MCP_ERROR".into(),
                    message,
                }),
            },
            TuhucarError::InvalidArgs { message, suggestion } => ApiError {
                code: "INVALID_ARGS".into(),
                message,
                retryable: false,
                suggestion: Some(suggestion),
                upstream: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn car_not_found_converts_to_api_error() {
        let err = TuhucarError::CarNotFound {
            suggestion: "请提供更精确的车型描述".into(),
        };
        let api_err: ApiError = err.into();
        assert_eq!(api_err.code, "CAR_NOT_FOUND");
        assert!(!api_err.retryable);
        assert_eq!(api_err.suggestion.unwrap(), "请提供更精确的车型描述");
    }

    #[test]
    fn mcp_error_5xx_is_retryable() {
        let err = TuhucarError::McpError {
            code: 502,
            message: "Bad Gateway".into(),
        };
        let api_err: ApiError = err.into();
        assert!(api_err.retryable);
        assert_eq!(api_err.upstream.unwrap().status, 502);
    }

    #[test]
    fn mcp_error_4xx_is_not_retryable() {
        let err = TuhucarError::McpError {
            code: 400,
            message: "Bad Request".into(),
        };
        let api_err: ApiError = err.into();
        assert!(!api_err.retryable);
    }
}
