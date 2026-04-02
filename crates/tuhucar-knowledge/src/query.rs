use tuhucar_core::error::TuhucarError;
use tuhucar_core::mcp::McpClient;
use crate::models::KnowledgeQueryOutput;

pub async fn query_knowledge(
    client: &McpClient,
    car_id: &str,
    question: &str,
) -> Result<KnowledgeQueryOutput, TuhucarError> {
    let body = client
        .call_tool(
            "knowledge_query",
            serde_json::json!({ "car_id": car_id, "question": question }),
        )
        .await?;

    serde_json::from_str(&body).map_err(|e| TuhucarError::McpError {
        code: -1,
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
