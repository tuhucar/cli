use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use tuhucar_core::Render;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CarMatchInput {
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CarMatchResult {
    pub candidates: Vec<CarMatchCandidate>,
    pub total_count: usize,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct CarMatchCandidate {
    pub car_id: String,
    pub brand: String,
    pub series: String,
    pub year: String,
    pub displacement: String,
    pub model: String,
    pub confidence: f64,
}

impl Render for CarMatchResult {
    fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap()
    }

    fn to_markdown(&self) -> String {
        if self.candidates.is_empty() {
            return "No matching car models found.\n".to_string();
        }
        let mut out = format!("Found {} matching car model(s):\n\n", self.total_count);
        for (i, c) in self.candidates.iter().enumerate() {
            out.push_str(&format!(
                "{}. **{} {} {} {} {}** (confidence: {:.0}%)\n   ID: `{}`\n\n",
                i + 1, c.brand, c.series, c.year, c.displacement, c.model,
                c.confidence * 100.0, c.car_id
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn car_match_result_markdown_renders_candidates() {
        let result = CarMatchResult {
            total_count: 1,
            candidates: vec![CarMatchCandidate {
                car_id: "12345".into(),
                brand: "大众".into(),
                series: "朗逸".into(),
                year: "2024".into(),
                displacement: "1.5L".into(),
                model: "自动舒适版".into(),
                confidence: 0.95,
            }],
        };
        let md = result.to_markdown();
        assert!(md.contains("大众"));
        assert!(md.contains("95%"));
        assert!(md.contains("`12345`"));
    }

    #[test]
    fn car_match_empty_candidates_markdown() {
        let result = CarMatchResult {
            total_count: 0,
            candidates: vec![],
        };
        let md = result.to_markdown();
        assert_eq!(md, "No matching car models found.\n");
    }

    #[test]
    fn car_match_multiple_candidates_markdown() {
        let result = CarMatchResult {
            total_count: 2,
            candidates: vec![
                CarMatchCandidate {
                    car_id: "111".into(),
                    brand: "大众".into(),
                    series: "朗逸".into(),
                    year: "2024".into(),
                    displacement: "1.5L".into(),
                    model: "舒适版".into(),
                    confidence: 0.95,
                },
                CarMatchCandidate {
                    car_id: "222".into(),
                    brand: "大众".into(),
                    series: "朗逸".into(),
                    year: "2023".into(),
                    displacement: "1.5L".into(),
                    model: "豪华版".into(),
                    confidence: 0.80,
                },
            ],
        };
        let md = result.to_markdown();
        assert!(md.contains("Found 2 matching car model(s):"));
        assert!(md.contains("1. **大众 朗逸 2024 1.5L 舒适版**"));
        assert!(md.contains("2. **大众 朗逸 2023 1.5L 豪华版**"));
        assert!(md.contains("95%"));
        assert!(md.contains("80%"));
        assert!(md.contains("`111`"));
        assert!(md.contains("`222`"));
    }

    #[test]
    fn car_match_to_json_roundtrips() {
        let result = CarMatchResult {
            total_count: 1,
            candidates: vec![CarMatchCandidate {
                car_id: "123".into(),
                brand: "丰田".into(),
                series: "卡罗拉".into(),
                year: "2024".into(),
                displacement: "1.8L".into(),
                model: "混动版".into(),
                confidence: 0.99,
            }],
        };
        let json_str = result.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["total_count"], 1);
        assert_eq!(parsed["candidates"][0]["brand"], "丰田");
    }
}
