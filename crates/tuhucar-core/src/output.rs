use schemars::JsonSchema;
use serde::Serialize;
use crate::types::{OutputFormat, Response, Render};

pub fn format_response<T: Serialize + JsonSchema + Render>(
    resp: &Response<T>,
    format: OutputFormat,
) -> String {
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(resp).unwrap(),
        OutputFormat::Markdown => {
            let mut out = String::new();
            if let Some(data) = &resp.data {
                out.push_str(&data.to_markdown());
            }
            if let Some(err) = &resp.error {
                out.push_str(&format!("**Error [{}]:** {}\n", err.code, err.message));
                if let Some(suggestion) = &err.suggestion {
                    out.push_str(&format!("\n> {}\n", suggestion));
                }
            }
            if let Some(meta) = &resp.meta {
                for notice in &meta.notices {
                    match notice {
                        crate::types::Notice::Update { message, .. } => {
                            out.push_str(&format!("\n<!-- {} -->\n", message));
                        }
                    }
                }
            }
            out
        }
    }
}
