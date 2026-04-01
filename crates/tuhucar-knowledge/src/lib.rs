pub mod models;
pub mod query;

use async_trait::async_trait;
use tuhucar_core::command::Command;
use tuhucar_core::ErrorSchemaEntry;
use tuhucar_core::error::TuhucarError;
use tuhucar_core::http::HttpClient;
use crate::models::{KnowledgeQueryInput, KnowledgeQueryOutput};

pub struct KnowledgeCommand {
    client: HttpClient,
}

impl KnowledgeCommand {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
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
                code: "NETWORK_ERROR".into(),
                description: "网络连接失败".into(),
                retryable: true,
            },
            ErrorSchemaEntry {
                code: "API_ERROR".into(),
                description: "后端服务错误".into(),
                retryable: false,
            },
        ]
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, TuhucarError> {
        query::query_knowledge(&self.client, &input.car_id, &input.question).await
    }
}
