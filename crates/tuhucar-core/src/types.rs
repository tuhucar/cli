use serde::Serialize;
use schemars::JsonSchema;

use crate::error::ApiError;

#[derive(Debug, Serialize, JsonSchema)]
pub struct Response<T: Serialize + JsonSchema> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
}

impl<T: Serialize + JsonSchema> Response<T> {
    pub fn success(data: T, meta: Option<ResponseMeta>) -> Self {
        Self { data: Some(data), error: None, meta }
    }

    pub fn error(err: ApiError, meta: Option<ResponseMeta>) -> Self {
        Self { data: None, error: Some(err), meta }
    }
}

#[derive(Debug, Serialize, JsonSchema, Clone)]
pub struct ResponseMeta {
    pub version: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notices: Vec<Notice>,
}

#[derive(Debug, Serialize, JsonSchema, Clone)]
#[serde(tag = "type")]
pub enum Notice {
    #[serde(rename = "update")]
    Update {
        current: String,
        latest: String,
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    Json,
    Markdown,
}

impl OutputFormat {
    pub fn from_str_opt(s: &str) -> Option<Self> {
        match s {
            "json" => Some(Self::Json),
            "markdown" => Some(Self::Markdown),
            _ => None,
        }
    }
}

pub trait Render {
    fn to_json(&self) -> String;
    fn to_markdown(&self) -> String;
}

#[derive(Debug, Serialize, JsonSchema)]
pub struct CommandSchema {
    pub name: String,
    pub description: String,
    pub input: serde_json::Value,
    pub wire_output: serde_json::Value,
    pub errors: Vec<ErrorSchemaEntry>,
}

#[derive(Debug, Serialize, JsonSchema, Clone)]
pub struct ErrorSchemaEntry {
    pub code: String,
    pub description: String,
    pub retryable: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_response_serializes_without_error() {
        let resp = Response::success("hello".to_string(), None);
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["data"], "hello");
        assert!(json.get("error").is_none());
        assert!(json.get("meta").is_none());
    }

    #[test]
    fn error_response_serializes_without_data() {
        let api_err = ApiError {
            code: "CAR_NOT_FOUND".into(),
            message: "Not found".into(),
            retryable: false,
            suggestion: Some("Try again".into()),
            upstream: None,
        };
        let resp: Response<String> = Response::error(api_err, None);
        let json = serde_json::to_value(&resp).unwrap();
        assert!(json.get("data").is_none());
        assert_eq!(json["error"]["code"], "CAR_NOT_FOUND");
    }

    #[test]
    fn response_with_meta_and_notice() {
        let meta = ResponseMeta {
            version: "0.1.0".into(),
            notices: vec![Notice::Update {
                current: "0.1.0".into(),
                latest: "0.2.0".into(),
                message: "Update available".into(),
            }],
        };
        let resp = Response::success(42u32, Some(meta));
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["data"], 42);
        assert_eq!(json["meta"]["version"], "0.1.0");
        assert_eq!(json["meta"]["notices"][0]["type"], "update");
    }

    #[test]
    fn wire_output_schema_includes_envelope() {
        let schema = schemars::schema_for!(Response<String>);
        let json = serde_json::to_value(&schema).unwrap();
        let props = &json["properties"];
        assert!(props.get("data").is_some());
        assert!(props.get("error").is_some());
        assert!(props.get("meta").is_some());
    }
}
