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
    id: Option<u64>,
    method: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: Option<u64>,
    result: Option<serde_json::Value>,
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
struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    protocol_version: &'static str,
    capabilities: serde_json::Value,
    #[serde(rename = "clientInfo")]
    client_info: ClientInfo,
}

#[derive(Serialize)]
struct ClientInfo {
    name: &'static str,
    version: &'static str,
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

impl McpClient {
    /// Connect to an MCP server: send initialize + initialized notification.
    pub async fn connect(config: &Config) -> Result<Self, TuhucarError> {
        let timeout = Duration::from_secs(config.api.timeout);
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to build HTTP client");

        let mut mcp = Self {
            client,
            endpoint: config.api.endpoint.clone(),
            next_id: AtomicU64::new(1),
            session_id: None,
        };

        mcp.initialize().await?;
        Ok(mcp)
    }

    async fn initialize(&mut self) -> Result<(), TuhucarError> {
        let params = InitializeParams {
            protocol_version: "2025-03-26",
            capabilities: serde_json::json!({}),
            client_info: ClientInfo {
                name: "tuhucar-cli",
                version: env!("CARGO_PKG_VERSION"),
            },
        };

        let (resp, session_id) = self
            .send_request("initialize", Some(serde_json::to_value(params).unwrap()))
            .await?;

        if let Some(sid) = session_id {
            self.session_id = Some(sid);
        }

        // Verify server supports tools
        if let Some(result) = &resp.result {
            let caps = &result["capabilities"];
            if caps.get("tools").is_none() {
                return Err(TuhucarError::McpError {
                    code: -1,
                    message: "MCP server does not support tools capability".into(),
                });
            }
        }

        // Send initialized notification (no id, no response expected)
        self.send_notification("notifications/initialized", None)
            .await?;

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
            .send_request(
                "tools/call",
                Some(serde_json::to_value(params).unwrap()),
            )
            .await?;

        if let Some(err) = resp.error {
            return Err(TuhucarError::McpError {
                code: err.code,
                message: err.message,
            });
        }

        let result: ToolCallResult = serde_json::from_value(
            resp.result.ok_or_else(|| TuhucarError::McpError {
                code: -1,
                message: "Empty result from tools/call".into(),
            })?,
        )
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
            id: Some(id),
            method,
            params,
        };

        let mut http_req = self
            .client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        if let Some(sid) = &self.session_id {
            http_req = http_req.header("Mcp-Session-Id", sid);
        }

        let http_resp = http_req
            .json(&req)
            .send()
            .await?;

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

        let rpc_resp: JsonRpcResponse = serde_json::from_str(&body).map_err(|e| {
            TuhucarError::McpError {
                code: -1,
                message: format!("Failed to parse JSON-RPC response: {}", e),
            }
        })?;

        if let Some(err) = &rpc_resp.error {
            return Err(TuhucarError::McpError {
                code: err.code,
                message: err.message.clone(),
            });
        }

        Ok((rpc_resp, session_id))
    }

    async fn send_notification(
        &self,
        method: &str,
        params: Option<serde_json::Value>,
    ) -> Result<(), TuhucarError> {
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id: None,
            method,
            params,
        };

        let mut http_req = self
            .client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json");

        if let Some(sid) = &self.session_id {
            http_req = http_req.header("Mcp-Session-Id", sid);
        }

        let resp = http_req.json(&req).send().await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(TuhucarError::McpError {
                code: status.as_u16() as i64,
                message: format!("Notification failed HTTP {}: {}", status.as_u16(), body),
            });
        }

        Ok(())
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_rpc_request_serializes_correctly() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id: Some(1),
            method: "tools/call",
            params: Some(serde_json::json!({"name": "car_match"})),
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert_eq!(json["method"], "tools/call");
    }

    #[test]
    fn notification_has_no_id() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id: None,
            method: "notifications/initialized",
            params: None,
        };
        let json = serde_json::to_value(&req).unwrap();
        assert!(json.get("id").is_none());
        assert!(json.get("params").is_none());
    }
}
