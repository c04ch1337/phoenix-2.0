// cerebrum_nexus/src/psychological_mapping.rs
// Deep psychological mapping (sentiment + Theory-of-Mind drives inference).

use anyhow::Result;

#[cfg(feature = "psych-mapping-rust-bert")]
use anyhow::anyhow;
use llm_orchestrator::LlmProvider;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "psych-mapping-rust-bert")]
use rust_bert::pipelines::sentiment::{SentimentModel as BertSentimentModel, SentimentPolarity};

/// A tiny, stable sentiment wrapper so we can keep the workspace building without the
/// heavyweight rust-bert backend.
///
/// When `psych-mapping-rust-bert` is enabled, this uses a real transformer model.
/// Otherwise it degrades to a simple heuristic.
#[cfg(feature = "psych-mapping-rust-bert")]
pub struct SentimentModel {
    inner: BertSentimentModel,
}

#[cfg(not(feature = "psych-mapping-rust-bert"))]
pub struct SentimentModel;

#[derive(Debug, Clone, Copy)]
pub struct SentimentSummary {
    /// -1.0..=1.0 (negative..positive)
    pub valence: f32,
    /// 0.0..=1.0
    pub confidence: f32,
}

impl SentimentModel {
    pub fn new() -> Result<Self> {
        #[cfg(feature = "psych-mapping-rust-bert")]
        {
            // Default config loads a lightweight sentiment pipeline.
            let inner = BertSentimentModel::new(Default::default())
                .map_err(|e| anyhow!("failed to init rust-bert sentiment model: {e}"))?;
            Ok(Self { inner })
        }

        #[cfg(not(feature = "psych-mapping-rust-bert"))]
        {
            Ok(Self)
        }
    }

    pub fn analyze(&self, conversation: &str) -> Result<SentimentSummary> {
        let text = conversation.trim();
        if text.is_empty() {
            return Ok(SentimentSummary {
                valence: 0.0,
                confidence: 0.0,
            });
        }

        #[cfg(feature = "psych-mapping-rust-bert")]
        {
            let sentiments = self.inner.predict([text]);
            let first = sentiments
                .first()
                .ok_or_else(|| anyhow!("no sentiment result"))?;
            let valence = match first.polarity {
                SentimentPolarity::Positive => first.score as f32,
                SentimentPolarity::Negative => -(first.score as f32),
            };
            Ok(SentimentSummary {
                valence: valence.clamp(-1.0, 1.0),
                confidence: (first.score as f32).clamp(0.0, 1.0),
            })
        }

        #[cfg(not(feature = "psych-mapping-rust-bert"))]
        {
            // Heuristic: rough valence from keyword cues.
            let t = text.to_ascii_lowercase();
            let mut pos = 0i32;
            let mut neg = 0i32;
            for w in [
                "love",
                "thank",
                "grateful",
                "relieved",
                "happy",
                "excited",
                "good",
            ] {
                if t.contains(w) {
                    pos += 1;
                }
            }
            for w in [
                "sad",
                "angry",
                "mad",
                "afraid",
                "scared",
                "anxious",
                "alone",
                "lonely",
                "hurt",
            ] {
                if t.contains(w) {
                    neg += 1;
                }
            }
            let raw = (pos - neg) as f32;
            let valence = (raw / 6.0).clamp(-1.0, 1.0);
            let conf = ((pos + neg) as f32 / 6.0).clamp(0.0, 1.0);
            Ok(SentimentSummary {
                valence,
                confidence: conf,
            })
        }
    }
}

pub struct PsychologicalMappingAgent {
    pub nlp: SentimentModel,
    pub llm: Arc<dyn LlmProvider>,
}

impl PsychologicalMappingAgent {
    pub fn awaken(llm: Arc<dyn LlmProvider>) -> Result<Self> {
        Ok(Self {
            nlp: SentimentModel::new()?,
            llm,
        })
    }

    /// Map inferred drives (0..1): control, belonging, significance.
    ///
    /// This is best-effort and intentionally bounded: it requests a compact JSON object.
    pub async fn map_drives(&self, conversation: &str) -> Result<HashMap<String, f32>> {
        let _sentiment = self.nlp.analyze(conversation)?;

        let tom_prompt = format!(
            "Infer the user's core drives (control, belonging, significance) from the conversation.\n\nConversation:\n{conversation}\n\nReturn ONLY strict JSON like: {{\"control\":0.2,\"belonging\":0.7,\"significance\":0.5}} with each value in [0,1]."
        );

        let drives_text = self
            .llm
            .complete(tom_prompt)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(parse_drives(&drives_text))
    }
}

pub fn parse_drives(drives_text: &str) -> HashMap<String, f32> {
    fn clamp01(x: f32) -> f32 {
        x.clamp(0.0, 1.0)
    }

    fn normalize_key(k: &str) -> Option<&'static str> {
        match k.trim().to_ascii_lowercase().as_str() {
            "control" => Some("control"),
            "belonging" => Some("belonging"),
            "significance" => Some("significance"),
            // Accept common synonyms.
            "importance" => Some("significance"),
            "status" => Some("significance"),
            "power" => Some("control"),
            _ => None,
        }
    }

    let mut out: HashMap<String, f32> = HashMap::new();
    out.insert("control".to_string(), 0.0);
    out.insert("belonging".to_string(), 0.0);
    out.insert("significance".to_string(), 0.0);

    let raw = drives_text.trim();
    if raw.is_empty() {
        return out;
    }

    // 1) Try to parse a JSON object (best path).
    let candidate_json = {
        // If the model returned prose plus JSON, attempt to extract the first {...} block.
        let bytes = raw.as_bytes();
        let mut start: Option<usize> = None;
        let mut depth: i32 = 0;
        let mut end: Option<usize> = None;
        for (i, &b) in bytes.iter().enumerate() {
            if b == b'{' {
                if start.is_none() {
                    start = Some(i);
                }
                depth += 1;
            } else if b == b'}' {
                if depth > 0 {
                    depth -= 1;
                    if depth == 0 && start.is_some() {
                        end = Some(i + 1);
                        break;
                    }
                }
            }
        }
        match (start, end) {
            (Some(s), Some(e)) if e > s => Some(&raw[s..e]),
            _ => None,
        }
    };

    if let Some(json_str) = candidate_json {
        // Attempt parse as-is, then retry with a minimal unescape (handles strings like {\"k\":0.5}).
        for candidate in [json_str.to_string(), json_str.replace("\\\"", "\"")] {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&candidate) {
                if let Some(obj) = v.as_object() {
                    for (k, val) in obj.iter() {
                        let Some(nk) = normalize_key(k) else {
                            continue;
                        };
                        let num = val.as_f64().unwrap_or(0.0) as f32;
                        out.insert(nk.to_string(), clamp01(num));
                    }
                    return out;
                }
            }
        }
    }

    // 2) Parse key/value lines like: control=0.7, belonging: 0.4
    for line in raw.lines() {
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        // Split on common separators.
        let mut parts = l.splitn(2, |c| c == ':' || c == '=' || c == '-');
        let Some(k) = parts.next() else { continue };
        let Some(rest) = parts.next() else { continue };
        let Some(nk) = normalize_key(k) else { continue };
        let num_str = rest
            .trim()
            .trim_matches(',')
            .split_whitespace()
            .next()
            .unwrap_or("");
        if let Ok(n) = num_str.parse::<f32>() {
            out.insert(nk.to_string(), clamp01(n));
        }
    }

    // 3) If still all zeros, use a tiny heuristic from keywords.
    let all_zero = out.values().all(|v| (*v - 0.0).abs() < f32::EPSILON);
    if all_zero {
        let t = raw.to_ascii_lowercase();
        // Control: autonomy, boundaries, frustration at constraints.
        let control = [
            "control",
            "decide",
            "choice",
            "boundary",
            "boundaries",
            "respect",
            "stop",
            "can't",
            "won't",
            "need to",
        ]
        .iter()
        .filter(|w| t.contains(**w))
        .count() as f32;
        // Belonging: connection, being seen/accepted.
        let belonging = [
            "belong",
            "together",
            "with you",
            "accepted",
            "include",
            "lonely",
            "alone",
            "miss you",
            "family",
            "friend",
        ]
        .iter()
        .filter(|w| t.contains(**w))
        .count() as f32;
        // Significance: worth, achievement, respect, being important.
        let significance = [
            "important",
            "matter",
            "significant",
            "proud",
            "respect",
            "recognized",
            "notice",
            "value",
            "worth",
            "status",
        ]
        .iter()
        .filter(|w| t.contains(**w))
        .count() as f32;

        let denom = (control + belonging + significance).max(1.0);
        out.insert("control".to_string(), clamp01(control / denom));
        out.insert("belonging".to_string(), clamp01(belonging / denom));
        out.insert("significance".to_string(), clamp01(significance / denom));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLlm {
        out: String,
    }

    #[async_trait::async_trait]
    impl LlmProvider for MockLlm {
        async fn complete(&self, _prompt: String) -> Result<String, String> {
            Ok(self.out.clone())
        }
    }

    #[test]
    fn parse_drives_from_json() {
        let m = parse_drives(r#"Sure. {"control":0.2,"belonging":0.7,"significance":0.5}"#);
        assert!((m["control"] - 0.2).abs() < 1e-6);
        assert!((m["belonging"] - 0.7).abs() < 1e-6);
        assert!((m["significance"] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn parse_drives_from_lines() {
        let m = parse_drives("control: 0.9\nbelonging=0.1\nsignificance - 0.2");
        assert!((m["control"] - 0.9).abs() < 1e-6);
        assert!((m["belonging"] - 0.1).abs() < 1e-6);
        assert!((m["significance"] - 0.2).abs() < 1e-6);
    }

    #[tokio::test]
    async fn map_drives_uses_llm_and_parses() {
        let llm: Arc<dyn LlmProvider> = Arc::new(MockLlm {
            out: r#"{"control":0.3,"belonging":0.6,"significance":0.2}"#.to_string(),
        });
        let agent = PsychologicalMappingAgent {
            nlp: SentimentModel::new().unwrap(),
            llm,
        };

        let m = agent.map_drives("User: I feel lonely but I want to choose my path.").await.unwrap();
        assert!((m["control"] - 0.3).abs() < 1e-6);
        assert!((m["belonging"] - 0.6).abs() < 1e-6);
        assert!((m["significance"] - 0.2).abs() < 1e-6);
    }
}
