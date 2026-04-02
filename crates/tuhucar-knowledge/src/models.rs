use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use tuhucar_core::Render;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeQueryInput {
    pub car_id: String,
    pub question: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeQueryOutput {
    pub answer: String,
    pub links: Vec<ExternalLink>,
    #[serde(default)]
    pub related_questions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct ExternalLink {
    pub title: String,
    pub url: String,
    pub link_type: LinkType,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub enum LinkType {
    MiniProgram,
    H5,
}

impl Render for KnowledgeQueryOutput {
    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    fn to_markdown(&self) -> String {
        let mut out = format!("{}\n", self.answer);
        if !self.links.is_empty() {
            out.push_str("\n**相关链接：**\n");
            for link in &self.links {
                let badge = match link.link_type {
                    LinkType::MiniProgram => "[小程序]",
                    LinkType::H5 => "[H5]",
                };
                out.push_str(&format!("- {} [{}]({})\n", badge, link.title, link.url));
            }
        }
        if !self.related_questions.is_empty() {
            out.push_str("\n**相关问题：**\n");
            for q in &self.related_questions {
                out.push_str(&format!("- {}\n", q));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knowledge_output_markdown_renders_links() {
        let output = KnowledgeQueryOutput {
            answer: "建议每5000公里更换一次机油".into(),
            links: vec![ExternalLink {
                title: "预约保养".into(),
                url: "https://m.tuhu.cn/maintenance".into(),
                link_type: LinkType::H5,
            }],
            related_questions: vec!["机油品牌推荐".into()],
        };
        let md = output.to_markdown();
        assert!(md.contains("5000公里"));
        assert!(md.contains("[H5]"));
        assert!(md.contains("机油品牌推荐"));
    }

    #[test]
    fn knowledge_output_miniprogram_badge() {
        let output = KnowledgeQueryOutput {
            answer: "答案".into(),
            links: vec![ExternalLink {
                title: "小程序入口".into(),
                url: "weixin://miniprogram/abc".into(),
                link_type: LinkType::MiniProgram,
            }],
            related_questions: vec![],
        };
        let md = output.to_markdown();
        assert!(md.contains("[小程序]"));
        assert!(md.contains("[小程序入口](weixin://miniprogram/abc)"));
    }

    #[test]
    fn knowledge_output_no_links_no_questions() {
        let output = KnowledgeQueryOutput {
            answer: "简单答案".into(),
            links: vec![],
            related_questions: vec![],
        };
        let md = output.to_markdown();
        assert_eq!(md, "简单答案\n");
        assert!(!md.contains("相关链接"));
        assert!(!md.contains("相关问题"));
    }

    #[test]
    fn knowledge_output_to_json_roundtrips() {
        let output = KnowledgeQueryOutput {
            answer: "每5000公里".into(),
            links: vec![ExternalLink {
                title: "t".into(),
                url: "u".into(),
                link_type: LinkType::H5,
            }],
            related_questions: vec!["q1".into()],
        };
        let json_str = output.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["answer"], "每5000公里");
        assert_eq!(parsed["links"][0]["link_type"], "H5");
    }
}
