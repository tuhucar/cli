use tuhucar_core::error::TuhucarError;
use tuhucar_core::http::HttpClient;
use crate::models::KnowledgeQueryOutput;

pub async fn query_knowledge(
    client: &HttpClient,
    car_id: &str,
    question: &str,
) -> Result<KnowledgeQueryOutput, TuhucarError> {
    let body = client
        .get("/api/v1/knowledge/query", &[("car_id", car_id), ("q", question)])
        .await?;

    serde_json::from_str(&body).map_err(|e| TuhucarError::ApiError {
        status: 200,
        code: "PARSE_ERROR".into(),
        message: format!("Failed to parse knowledge response: {}", e),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_json_parses_knowledge_output() {
        let json = r#"{
            "answer": "每5000公里更换",
            "links": [{"title": "预约", "url": "https://m.tuhu.cn", "link_type": "H5"}],
            "related_questions": ["什么机油好"]
        }"#;
        let output: KnowledgeQueryOutput = serde_json::from_str(json).unwrap();
        assert_eq!(output.links.len(), 1);
    }
}
