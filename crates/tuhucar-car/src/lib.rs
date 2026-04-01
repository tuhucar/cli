pub mod matcher;
pub mod models;

use async_trait::async_trait;
use tuhucar_core::command::Command;
use tuhucar_core::ErrorSchemaEntry;
use tuhucar_core::error::TuhucarError;
use tuhucar_core::http::HttpClient;
use crate::models::{CarMatchInput, CarMatchResult};

pub struct CarCommand {
    client: HttpClient,
}

impl CarCommand {
    pub fn new(client: HttpClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl Command for CarCommand {
    type Input = CarMatchInput;
    type Output = CarMatchResult;

    fn name(&self) -> &str { "car.match" }
    fn description(&self) -> &str { "模糊匹配五级车型" }

    fn error_schemas(&self) -> Vec<ErrorSchemaEntry> {
        vec![
            ErrorSchemaEntry {
                code: "CAR_NOT_FOUND".into(),
                description: "未找到匹配的车型".into(),
                retryable: false,
            },
            ErrorSchemaEntry {
                code: "NETWORK_ERROR".into(),
                description: "网络连接失败".into(),
                retryable: true,
            },
        ]
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, TuhucarError> {
        matcher::match_car(&self.client, &input.query).await
    }
}
