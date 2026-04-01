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
}
