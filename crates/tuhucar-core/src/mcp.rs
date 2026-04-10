use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::config::Config;
use crate::error::TuhucarError;

#[derive(Serialize)]
struct JsonRpcRequest<'a> {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    method: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    #[serde(default)]
    jsonrpc: String,
    #[allow(dead_code)]
    #[serde(default)]
    id: Option<serde_json::Value>,
    result: Option<serde_json::Value>,
    #[serde(default)]
    error: Option<JsonRpcError>,
}

#[derive(Deserialize, Debug)]
struct JsonRpcError {
    code: i64,
    message: String,
    #[allow(dead_code)]
    data: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct ToolCallParams {
    name: String,
    arguments: serde_json::Value,
}

#[derive(Deserialize)]
struct ToolCallResult {
    content: Vec<ToolContent>,
    #[serde(rename = "isError", default)]
    is_error: bool,
}

#[derive(Deserialize)]
struct ToolContent {
    #[allow(dead_code)]
    r#type: String,
    text: String,
}

pub struct McpClient {
    client: Client,
    endpoint: String,
    next_id: AtomicU64,
    session_id: Option<String>,
}

fn parse_jsonrpc_body(body: &str) -> Result<JsonRpcResponse, TuhucarError> {
    let trimmed = body.trim_start();
    // Try plain JSON first.
    if trimmed.starts_with('{') {
        return serde_json::from_str(trimmed).map_err(|e| TuhucarError::McpError {
            code: -1,
            message: format!("Failed to parse JSON-RPC response: {}: {}", e, body),
        });
    }
    // Otherwise, parse as SSE: scan `data:` lines, return the last one whose
    // payload parses as a JSON-RPC envelope containing a result or error.
    let mut last: Option<JsonRpcResponse> = None;
    for line in body.lines() {
        let payload = match line.strip_prefix("data:") {
            Some(p) => p.trim(),
            None => continue,
        };
        if payload.is_empty() || payload == "[DONE]" {
            continue;
        }
        if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(payload) {
            // Skip progress-only frames.
            let is_progress = resp
                .result
                .as_ref()
                .and_then(|r| r.get("progress"))
                .is_some()
                && resp
                    .result
                    .as_ref()
                    .and_then(|r| r.get("content"))
                    .is_none();
            if is_progress {
                continue;
            }
            last = Some(resp);
        }
    }
    last.ok_or_else(|| TuhucarError::McpError {
        code: -1,
        message: format!("No JSON-RPC payload found in SSE stream: {}", body),
    })
}

impl McpClient {
    /// Connect to an MCP server: send initialize + initialized notification.
    pub async fn connect(config: &Config) -> Result<Self, TuhucarError> {
        let timeout = Duration::from_secs(config.api.timeout);
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to build HTTP client");

        // Allow runtime override (e.g. dev pointing at a test gateway) without
        // editing the on-disk config file.
        let endpoint = std::env::var("TUHUCAR_ENDPOINT")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| config.api.endpoint.clone());

        let mut mcp = Self {
            client,
            endpoint,
            next_id: AtomicU64::new(1),
            session_id: None,
        };

        mcp.initialize().await?;
        Ok(mcp)
    }

    async fn initialize(&mut self) -> Result<(), TuhucarError> {
        // Per gateway protocol, `initialize` is sent without params; the
        // session id is returned in the response body.
        let (resp, header_session_id) = self.send_request("initialize", None).await?;

        if let Some(sid) = header_session_id {
            self.session_id = Some(sid);
        }

        if let Some(result) = &resp.result {
            if let Some(sid) = result.get("sessionId").and_then(|v| v.as_str()) {
                self.session_id = Some(sid.to_string());
            }
            if result
                .get("capabilities")
                .and_then(|c| c.get("tools"))
                .is_none()
            {
                return Err(TuhucarError::McpError {
                    code: -1,
                    message: "MCP server does not advertise tools capability".into(),
                });
            }
        }

        if self.session_id.is_none() {
            return Err(TuhucarError::McpError {
                code: -1,
                message: "MCP server did not return a sessionId".into(),
            });
        }

        Ok(())
    }

    /// Call an MCP tool and return the text result.
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: serde_json::Value,
    ) -> Result<String, TuhucarError> {
        let params = ToolCallParams {
            name: tool_name.to_string(),
            arguments,
        };

        let (resp, _) = self
            .send_request("tools/call", Some(serde_json::to_value(params).unwrap()))
            .await?;

        if let Some(err) = resp.error {
            return Err(TuhucarError::McpError {
                code: err.code,
                message: err.message,
            });
        }

        let result: ToolCallResult =
            serde_json::from_value(resp.result.ok_or_else(|| TuhucarError::McpError {
                code: -1,
                message: "Empty result from tools/call".into(),
            })?)
            .map_err(|e| TuhucarError::McpError {
                code: -1,
                message: format!("Failed to parse tool result: {}", e),
            })?;

        if result.is_error {
            let msg = result
                .content
                .first()
                .map(|c| c.text.clone())
                .unwrap_or_else(|| "Tool returned an error".into());
            return Err(TuhucarError::McpError {
                code: -1,
                message: msg,
            });
        }

        result
            .content
            .into_iter()
            .find(|c| c.r#type == "text")
            .map(|c| c.text)
            .ok_or_else(|| TuhucarError::McpError {
                code: -1,
                message: "No text content in tool result".into(),
            })
    }

    async fn send_request(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<(JsonRpcResponse, Option<String>), TuhucarError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id: Some(id.to_string()),
            method,
            params,
        };

        let mut http_req = self
            .client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header("Connection", "keep-alive");

        if let Some(sid) = &self.session_id {
            http_req = http_req.header("Mcp-Session-Id", sid);
        }

        let http_resp = http_req.json(&req).send().await?;

        let session_id = http_resp
            .headers()
            .get("Mcp-Session-Id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let status = http_resp.status();
        let body = http_resp.text().await?;

        if !status.is_success() {
            return Err(TuhucarError::McpError {
                code: status.as_u16() as i64,
                message: format!("MCP server returned HTTP {}: {}", status.as_u16(), body),
            });
        }

        let rpc_resp = parse_jsonrpc_body(&body)?;

        if let Some(err) = &rpc_resp.error {
            return Err(TuhucarError::McpError {
                code: err.code,
                message: err.message.clone(),
            });
        }

        Ok((rpc_resp, session_id))
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_rpc_request_serializes_with_string_id() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id: Some("1".into()),
            method: "tools/call",
            params: Some(serde_json::json!({"name": "x"})),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], "1");
        assert_eq!(json["method"], "tools/call");
    }

    #[test]
    fn parses_plain_json_body() {
        let body = r#"{"jsonrpc":"2.0","id":"1","result":{"sessionId":"abc","capabilities":{"tools":{}}}}"#;
        let resp = parse_jsonrpc_body(body).unwrap();
        assert_eq!(
            resp.result
                .unwrap()
                .get("sessionId")
                .unwrap()
                .as_str()
                .unwrap(),
            "abc"
        );
    }

    #[test]
    fn parses_sse_body_skipping_progress_and_done() {
        let body = "id: 1\nevent: message\ndata: {\"result\":{\"progress\":1.0,\"message\":\"x\"},\"id\":\"1\",\"jsonrpc\":\"2.0\"}\n\nid: 2\nevent: message\ndata: {\"result\":{\"isError\":false,\"content\":[{\"type\":\"text\",\"text\":\"hello\"}]},\"id\":\"1\",\"jsonrpc\":\"2.0\"}\n\ndata: [DONE]\n";
        let resp = parse_jsonrpc_body(body).unwrap();
        let result = resp.result.unwrap();
        assert!(result.get("content").is_some());
        assert_eq!(result["content"][0]["text"], "hello");
    }
}
