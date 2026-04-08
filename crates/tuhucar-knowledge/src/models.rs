use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tuhucar_core::Render;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeQueryInput {
    /// 用户问题
    pub question: String,
    /// 可选会话 ID（用于多轮对话），不传则自动生成
    #[serde(default)]
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeQueryOutput {
    /// 模型回答
    pub reply: String,
    /// 会话 ID
    pub session_id: String,
    /// 消息 ID
    pub msg_id: String,
}

impl Render for KnowledgeQueryOutput {
    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    fn to_markdown(&self) -> String {
        self.reply.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knowledge_output_markdown_returns_reply() {
        let output = KnowledgeQueryOutput {
            reply: "建议每5000公里更换一次机油".into(),
            session_id: "s1".into(),
            msg_id: "m1".into(),
        };
        assert!(output.to_markdown().contains("5000公里"));
    }

    #[test]
    fn knowledge_output_markdown_is_plain_reply() {
        let output = KnowledgeQueryOutput {
            reply: "简单答案".into(),
            session_id: "s".into(),
            msg_id: "m".into(),
        };
        // Markdown rendering is just the reply, untouched.
        assert_eq!(output.to_markdown(), "简单答案");
    }

    #[test]
    fn knowledge_output_to_json_roundtrips() {
        let output = KnowledgeQueryOutput {
            reply: "每5000公里".into(),
            session_id: "sess-1".into(),
            msg_id: "msg-1".into(),
        };
        let json_str = output.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["reply"], "每5000公里");
        assert_eq!(parsed["session_id"], "sess-1");
        assert_eq!(parsed["msg_id"], "msg-1");
    }
}
