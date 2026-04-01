use reqwest::Client;
use std::time::Duration;
use crate::config::Config;
use crate::error::TuhucarError;

pub struct HttpClient {
    client: Client,
    base_url: String,
    timeout: Duration,
}

impl HttpClient {
    pub fn new(config: &Config) -> Self {
        let timeout = Duration::from_secs(config.api.timeout);
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to build HTTP client");
        Self {
            client,
            base_url: config.api.base_url.clone(),
            timeout,
        }
    }

    pub async fn get(&self, path: &str, params: &[(&str, &str)]) -> Result<String, TuhucarError> {
        let url = format!("{}{}", self.base_url, path);
        match self.do_get(&url, params).await {
            Ok(body) => Ok(body),
            Err(e) if Self::is_retryable(&e) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
                self.do_get(&url, params).await
            }
            Err(e) => Err(e),
        }
    }

    async fn do_get(&self, url: &str, params: &[(&str, &str)]) -> Result<String, TuhucarError> {
        let resp = self
            .client
            .get(url)
            .query(params)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if status.is_success() {
            Ok(body)
        } else {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&body) {
                let code = parsed["code"].as_str().unwrap_or("UNKNOWN").to_string();
                let message = parsed["message"].as_str().unwrap_or(&body).to_string();
                Err(TuhucarError::ApiError {
                    status: status.as_u16(),
                    code,
                    message,
                })
            } else {
                Err(TuhucarError::ApiError {
                    status: status.as_u16(),
                    code: "UNKNOWN".into(),
                    message: body,
                })
            }
        }
    }

    fn is_retryable(err: &TuhucarError) -> bool {
        match err {
            TuhucarError::Network(e) => e.is_connect() || e.is_timeout(),
            _ => false,
        }
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_retryable_returns_false_for_api_error() {
        let err = TuhucarError::ApiError {
            status: 400,
            code: "BAD".into(),
            message: "bad".into(),
        };
        assert!(!HttpClient::is_retryable(&err));
    }
}
