pub mod models;
pub mod query;

use async_trait::async_trait;
use tuhucar_core::command::Command;
use tuhucar_core::ErrorSchemaEntry;
use tuhucar_core::error::TuhucarError;
use tuhucar_core::mcp::McpClient;
use crate::models::{KnowledgeQueryInput, KnowledgeQueryOutput};

pub struct KnowledgeCommand {
    client: McpClient,
}

impl KnowledgeCommand {
    pub fn new(client: McpClient) -> Self {
        Self { client }
    }

    /// Generate schema without requiring a live MCP connection.
    pub fn schema_static() -> tuhucar_core::CommandSchema {
        use tuhucar_core::types::CommandSchema;
        use tuhucar_core::Response;
        CommandSchema {
            name: "knowledge.query".to_string(),
            description: "查询养车知识".to_string(),
            input: serde_json::to_value(schemars::schema_for!(KnowledgeQueryInput)).unwrap(),
            wire_output: serde_json::to_value(
                schemars::schema_for!(Response<KnowledgeQueryOutput>),
            ).unwrap(),
            errors: vec![
                ErrorSchemaEntry {
                    code: "MCP_ERROR".into(),
                    description: "MCP 服务调用失败".into(),
                    retryable: true,
                },
            ],
        }
    }
}

#[async_trait]
impl Command for KnowledgeCommand {
    type Input = KnowledgeQueryInput;
    type Output = KnowledgeQueryOutput;

    fn name(&self) -> &str { "knowledge.query" }
    fn description(&self) -> &str { "查询养车知识" }

    fn error_schemas(&self) -> Vec<ErrorSchemaEntry> {
        vec![
            ErrorSchemaEntry {
                code: "MCP_ERROR".into(),
                description: "MCP 服务调用失败".into(),
                retryable: true,
            },
        ]
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, TuhucarError> {
        query::query_knowledge(&self.client, &input.question, input.session_id.as_deref()).await
    }
}
