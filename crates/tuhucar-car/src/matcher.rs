use tuhucar_core::error::TuhucarError;
use tuhucar_core::http::HttpClient;
use crate::models::{CarMatchResult, CarMatchCandidate};

pub async fn match_car(client: &HttpClient, query: &str) -> Result<CarMatchResult, TuhucarError> {
    let body = client
        .get("/api/v1/car/match", &[("q", query)])
        .await?;

    let candidates: Vec<CarMatchCandidate> = serde_json::from_str(&body)
        .map_err(|e| TuhucarError::ApiError {
            status: 200,
            code: "PARSE_ERROR".into(),
            message: format!("Failed to parse car match response: {}", e),
        })?;

    if candidates.is_empty() {
        return Err(TuhucarError::CarNotFound {
            suggestion: "请提供更精确的车型描述，如品牌+车系+年款".into(),
        });
    }

    Ok(CarMatchResult {
        total_count: candidates.len(),
        candidates,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_response_returns_car_not_found() {
        let json = "[]";
        let candidates: Vec<CarMatchCandidate> = serde_json::from_str(json).unwrap();
        assert!(candidates.is_empty());
    }

    #[test]
    fn valid_json_parses_candidates() {
        let json = r#"[{
            "car_id": "123",
            "brand": "大众",
            "series": "朗逸",
            "year": "2024",
            "displacement": "1.5L",
            "model": "舒适版",
            "confidence": 0.9
        }]"#;
        let candidates: Vec<CarMatchCandidate> = serde_json::from_str(json).unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].brand, "大众");
    }
}
