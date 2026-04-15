pub mod matcher;
pub mod models;

use crate::models::{CarMatchInput, CarMatchResult};
use async_trait::async_trait;
use tuhucar_core::command::Command;
use tuhucar_core::error::TuhucarError;
use tuhucar_core::mcp::McpClient;
use tuhucar_core::ErrorSchemaEntry;

pub struct CarCommand {
    client: McpClient,
}

impl CarCommand {
    pub fn new(client: McpClient) -> Self {
        Self { client }
    }

    /// Generate schema without requiring a live MCP connection.
    pub fn schema_static() -> tuhucar_core::CommandSchema {
        use tuhucar_core::types::CommandSchema;
        use tuhucar_core::Response;
        CommandSchema {
            name: "car.match".to_string(),
            description: "模糊匹配五级车型".to_string(),
            input: serde_json::to_value(schemars::schema_for!(CarMatchInput)).unwrap(),
            wire_output: serde_json::to_value(schemars::schema_for!(Response<CarMatchResult>)).unwrap(),
            errors: vec![
                ErrorSchemaEntry {
                    code: "CAR_NOT_FOUND".into(),
                    description: "未找到匹配的车型".into(),
                    retryable: false,
                },
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
impl Command for CarCommand {
    type Input = CarMatchInput;
    type Output = CarMatchResult;

    fn name(&self) -> &str {
        "car.match"
    }
    fn description(&self) -> &str {
        "模糊匹配五级车型"
    }

    fn error_schemas(&self) -> Vec<ErrorSchemaEntry> {
        vec![
            ErrorSchemaEntry {
                code: "CAR_NOT_FOUND".into(),
                description: "未找到匹配的车型".into(),
                retryable: false,
            },
            ErrorSchemaEntry {
                code: "MCP_ERROR".into(),
                description: "MCP 服务调用失败".into(),
                retryable: true,
            },
        ]
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, TuhucarError> {
        matcher::match_car(&self.client, &input.query).await
    }
}
