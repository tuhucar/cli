use crate::error::ApiError;
use crate::types::{OutputFormat, Render, Response};
use schemars::JsonSchema;
use serde::Serialize;

fn is_upstream_rpc_error(err: &ApiError) -> bool {
    err.code == "MCP_ERROR"
        && err
            .upstream
            .as_ref()
            .and_then(|upstream| upstream.rpc_code)
            .is_some()
}

pub fn format_markdown_error(err: &ApiError) -> String {
    let mut out = String::new();

    if is_upstream_rpc_error(err) {
        out.push_str(&err.message);
        out.push('\n');
    } else {
        out.push_str(&format!("Error [{}]: {}\n", err.code, err.message));
    }

    if let Some(suggestion) = &err.suggestion {
        out.push_str(&format!("\n  {}\n", suggestion));
    }

    out
}

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
                if is_upstream_rpc_error(err) {
                    out.push_str(&err.message);
                    out.push('\n');
                } else {
                    out.push_str(&format!("**Error [{}]:** {}\n", err.code, err.message));
                }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ApiError;
    use crate::types::{Notice, ResponseMeta};

    #[derive(Debug, Serialize, schemars::JsonSchema)]
    struct TestData {
        value: String,
    }

    impl Render for TestData {
        fn to_json(&self) -> String {
            serde_json::to_string(self).unwrap()
        }
        fn to_markdown(&self) -> String {
            format!("Value: {}\n", self.value)
        }
    }

    #[test]
    fn json_format_returns_valid_json() {
        let resp = Response::success(
            TestData {
                value: "hello".into(),
            },
            None,
        );
        let output = format_response(&resp, OutputFormat::Json);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["data"]["value"], "hello");
    }

    #[test]
    fn markdown_format_renders_data() {
        let resp = Response::success(
            TestData {
                value: "hello".into(),
            },
            None,
        );
        let output = format_response(&resp, OutputFormat::Markdown);
        assert!(output.contains("Value: hello"));
    }

    #[test]
    fn markdown_format_renders_error_with_suggestion() {
        let api_err = ApiError {
            code: "TEST_ERROR".into(),
            message: "Something went wrong".into(),
            retryable: false,
            suggestion: Some("Try again later".into()),
            upstream: None,
        };
        let resp: Response<TestData> = Response::error(api_err, None);
        let output = format_response(&resp, OutputFormat::Markdown);
        assert!(output.contains("**Error [TEST_ERROR]:** Something went wrong"));
        assert!(output.contains("> Try again later"));
    }

    #[test]
    fn markdown_format_renders_error_without_suggestion() {
        let api_err = ApiError {
            code: "NET_ERR".into(),
            message: "Network failed".into(),
            retryable: true,
            suggestion: None,
            upstream: None,
        };
        let resp: Response<TestData> = Response::error(api_err, None);
        let output = format_response(&resp, OutputFormat::Markdown);
        assert!(output.contains("**Error [NET_ERR]:** Network failed"));
        assert!(!output.contains(">"));
    }

    #[test]
    fn markdown_error_formatter_uses_plain_message_for_rpc_errors() {
        let api_err = ApiError {
            code: "MCP_ERROR".into(),
            message: "抱歉亲，今天忙疯了，明天再来询问吧".into(),
            retryable: false,
            suggestion: None,
            upstream: Some(crate::error::UpstreamError {
                status: None,
                rpc_code: Some(-32007),
                code: "MCP_ERROR".into(),
                message: "抱歉亲，今天忙疯了，明天再来询问吧".into(),
            }),
        };
        let output = format_markdown_error(&api_err);
        assert_eq!(output, "抱歉亲，今天忙疯了，明天再来询问吧\n");
    }

    #[test]
    fn markdown_format_renders_update_notice() {
        let meta = ResponseMeta {
            version: "0.1.0".into(),
            notices: vec![Notice::Update {
                current: "0.1.0".into(),
                latest: "0.2.0".into(),
                message: "Update available!".into(),
            }],
        };
        let resp = Response::success(TestData { value: "ok".into() }, Some(meta));
        let output = format_response(&resp, OutputFormat::Markdown);
        assert!(output.contains("Value: ok"));
        assert!(output.contains("<!-- Update available! -->"));
    }

    #[test]
    fn json_format_includes_meta() {
        let meta = ResponseMeta {
            version: "0.1.0".into(),
            notices: vec![Notice::Update {
                current: "0.1.0".into(),
                latest: "0.2.0".into(),
                message: "Update".into(),
            }],
        };
        let resp = Response::success(TestData { value: "ok".into() }, Some(meta));
        let output = format_response(&resp, OutputFormat::Json);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["meta"]["version"], "0.1.0");
        assert_eq!(parsed["meta"]["notices"][0]["type"], "update");
    }
}
