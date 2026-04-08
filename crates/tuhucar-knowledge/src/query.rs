use crate::models::KnowledgeQueryOutput;
use serde::Deserialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tuhucar_core::error::TuhucarError;
use tuhucar_core::mcp::McpClient;

static MSG_COUNTER: AtomicU64 = AtomicU64::new(1);

const TOOL_NAME: &str = "mkt-intelligent-skill-dialogue";

#[derive(Deserialize)]
struct GatewayEnvelope {
    code: i64,
    #[serde(default)]
    message: Option<String>,
    data: Option<GatewayData>,
}

#[derive(Deserialize)]
struct GatewayData {
    reply: String,
    #[serde(rename = "sessionId")]
    session_id: String,
    #[serde(rename = "msgId")]
    msg_id: String,
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub async fn query_knowledge(
    client: &McpClient,
    question: &str,
    session_id: Option<&str>,
) -> Result<KnowledgeQueryOutput, TuhucarError> {
    let now = now_millis();
    let session = session_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| now.to_string());
    // Gateway requires msgId to be a pure-digit string (Long → String).
    // Combine timestamp (ms) with a 3-digit zero-padded counter to stay unique
    // within the same millisecond without introducing non-digit characters.
    let counter = MSG_COUNTER.fetch_add(1, Ordering::Relaxed) % 1000;
    let msg_id = format!("{}{:03}", now, counter);

    let arguments = serde_json::json!({
        "sessionId": session,
        "msgId": msg_id,
        "query": [{
            "textFormat": "TXT",
            "text": question,
            "timeStamp": now,
        }],
    });

    let body = client.call_tool(TOOL_NAME, arguments).await?;

    let envelope: GatewayEnvelope =
        serde_json::from_str(&body).map_err(|e| TuhucarError::McpError {
            code: -1,
            message: format!("Failed to parse knowledge response: {}: {}", e, body),
        })?;

    if envelope.code != 10000 {
        return Err(TuhucarError::McpError {
            code: envelope.code,
            message: envelope
                .message
                .unwrap_or_else(|| "Knowledge gateway returned non-success code".into()),
        });
    }

    let data = envelope.data.ok_or_else(|| TuhucarError::McpError {
        code: -1,
        message: "Knowledge gateway returned no data".into(),
    })?;

    Ok(KnowledgeQueryOutput {
        reply: data.reply,
        session_id: data.session_id,
        msg_id: data.msg_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gateway_envelope() {
        let body = r#"{"code":10000,"data":{"reply":"换机油","sessionId":"s","msgId":"m"},"message":"ok"}"#;
        let env: GatewayEnvelope = serde_json::from_str(body).unwrap();
        assert_eq!(env.code, 10000);
        assert_eq!(env.data.unwrap().reply, "换机油");
    }
}
