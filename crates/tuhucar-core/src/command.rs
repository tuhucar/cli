use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Serialize;

use crate::types::{CommandSchema, ErrorSchemaEntry};
use crate::error::TuhucarError;
use crate::types::Response;

#[async_trait]
pub trait Command: Send + Sync {
    type Input: Send + JsonSchema;
    type Output: Serialize + JsonSchema;

    fn name(&self) -> &str;
    fn description(&self) -> &str;

    fn schema(&self) -> CommandSchema {
        CommandSchema {
            name: self.name().to_string(),
            description: self.description().to_string(),
            input: serde_json::to_value(schemars::schema_for!(Self::Input)).unwrap(),
            wire_output: serde_json::to_value(
                schemars::schema_for!(Response<Self::Output>),
            )
            .unwrap(),
            errors: self.error_schemas(),
        }
    }

    fn error_schemas(&self) -> Vec<ErrorSchemaEntry> {
        vec![]
    }

    async fn execute(&self, input: Self::Input) -> Result<Self::Output, TuhucarError>;
}
