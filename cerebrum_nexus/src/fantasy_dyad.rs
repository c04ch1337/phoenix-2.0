// cerebrum_nexus/src/fantasy_dyad.rs
// Fantasy Dyad Agent: relational co-adaptation (persona evolution + user drive model).

use anyhow::Result;
use llm_orchestrator::LlmProvider;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Compact drive mapping for the current interaction (0..=1 values).
///
/// Expected keys (best-effort): `control`, `belonging`, `significance`.
pub type DriveMap = HashMap<String, f32>;

fn clamp01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToneProfile {
    Neutral,
    Gentle,
    Grounded,
    Encouraging,
    Playful,
}

impl ToneProfile {
    fn as_str(&self) -> &'static str {
        match self {
            ToneProfile::Neutral => "neutral",
            ToneProfile::Gentle => "gentle",
            ToneProfile::Grounded => "grounded",
            ToneProfile::Encouraging => "encouraging",
            ToneProfile::Playful => "playful",
        }
    }
}

/// Mutable persona parameters (what changes turn-by-turn).
#[derive(Debug, Clone)]
pub struct PersonaState {
    pub tone: ToneProfile,
    /// 0..=1: warmth / reassurance.
    pub warmth: f32,
    /// 0..=1: directness / concision.
    pub directness: f32,
    /// 0..=1: playfulness / banter.
    pub playfulness: f32,
    /// 0..=1: autonomy support (options, consent, boundaries).
    pub autonomy_support: f32,
    /// 0..=1: affirmation / validation of competence & worth.
    pub affirmation: f32,
    /// Last computed rationale (debuggable, not required).
    pub last_rationale: Option<String>,
}

impl Default for PersonaState {
    fn default() -> Self {
        Self {
            tone: ToneProfile::Neutral,
            warmth: 0.55,
            directness: 0.55,
            playfulness: 0.35,
            autonomy_support: 0.55,
            affirmation: 0.55,
            last_rationale: None,
        }
    }
}

/// A lightweight user model: EMA of drives to smooth across turns.
#[derive(Debug, Clone)]
pub struct UserDriveMap {
    pub ema: DriveMap,
    pub alpha: f32,
    pub samples: u64,
}

impl Default for UserDriveMap {
    fn default() -> Self {
        let mut ema = DriveMap::new();
        ema.insert("control".to_string(), 0.0);
        ema.insert("belonging".to_string(), 0.0);
        ema.insert("significance".to_string(), 0.0);
        Self {
            ema,
            alpha: 0.30,
            samples: 0,
        }
    }
}

impl UserDriveMap {
    fn update_from(&mut self, drives: &DriveMap) {
        let a = clamp01(self.alpha);
        for k in ["control", "belonging", "significance"] {
            let x = drives.get(k).copied().unwrap_or(0.0);
            let prev = self.ema.get(k).copied().unwrap_or(0.0);
            let next = (a * clamp01(x)) + ((1.0 - a) * clamp01(prev));
            self.ema.insert(k.to_string(), clamp01(next));
        }
        self.samples = self.samples.saturating_add(1);
    }
}

/// FantasyDyadAgent: maintains a persona state and a smoothed user drive model.
///
/// This agent does not enforce safety. Callers should apply [`ethical_agent::EthicalAgent`]
/// vetoes before emitting output.
pub struct FantasyDyadAgent {
    pub persona: Mutex<PersonaState>,
    pub user_model: Mutex<UserDriveMap>,
    pub llm: Arc<dyn LlmProvider>,
}

impl FantasyDyadAgent {
    pub fn awaken(llm: Arc<dyn LlmProvider>) -> Self {
        Self {
            persona: Mutex::new(PersonaState::default()),
            user_model: Mutex::new(UserDriveMap::default()),
            llm,
        }
    }

    /// Generate a response with co-adaptation applied.
    pub async fn generate_response(&self, input: &str, mapping: &DriveMap) -> Result<String> {
        let persona_prompt = self.co_adapt_persona(mapping).await?;
        let prompt = format!(
            "{persona_prompt}\n\nUser: {input}\nAssistant:",
            persona_prompt = persona_prompt.trim_end(),
            input = input.trim()
        );
        self.llm
            .complete(prompt)
            .await
            .map_err(|e| anyhow::anyhow!(e))
    }

    /// Adjust tone based on inferred drives and return a prompt prefix that materializes
    /// the adapted persona.
    pub async fn co_adapt_persona(&self, drives: &DriveMap) -> Result<String> {
        // Update user model (EMA).
        let (control, belonging, significance) = {
            let mut um = self.user_model.lock().await;
            um.update_from(drives);
            (
                um.ema.get("control").copied().unwrap_or(0.0),
                um.ema.get("belonging").copied().unwrap_or(0.0),
                um.ema.get("significance").copied().unwrap_or(0.0),
            )
        };

        // Adapt persona state.
        let mut persona = self.persona.lock().await;
        persona.autonomy_support = clamp01(0.35 + 0.65 * control);
        persona.warmth = clamp01(0.35 + 0.65 * belonging);
        persona.affirmation = clamp01(0.25 + 0.75 * significance);

        // Directness increases with control (preference for agency/clarity) but softens slightly
        // with belonging (more relational padding).
        persona.directness = clamp01(0.45 + 0.55 * control - 0.15 * belonging);

        // Playfulness is mostly belonging-driven and dampened by high control (which can signal
        // boundary-setting / need for steadiness).
        persona.playfulness = clamp01(0.15 + 0.65 * belonging - 0.20 * control);

        persona.tone = if belonging >= 0.72 {
            if control >= 0.65 {
                ToneProfile::Grounded
            } else if persona.playfulness >= 0.55 {
                ToneProfile::Playful
            } else {
                ToneProfile::Gentle
            }
        } else if control >= 0.72 {
            ToneProfile::Grounded
        } else if significance >= 0.72 {
            ToneProfile::Encouraging
        } else {
            ToneProfile::Neutral
        };

        let rationale = format!(
            "drives_ema control={control:.2} belonging={belonging:.2} significance={significance:.2} -> tone={} warmth={:.2} directness={:.2} playfulness={:.2} autonomy={:.2} affirmation={:.2}",
            persona.tone.as_str(),
            persona.warmth,
            persona.directness,
            persona.playfulness,
            persona.autonomy_support,
            persona.affirmation
        );
        persona.last_rationale = Some(rationale);

        // Prompt prefix (system-like) used by the LLM completion API.
        let prompt = format!(
            "You are Phoenix (relational dyad mode). Co-adapt to the user's needs while staying safe and supportive.\n\nDYAD_ADAPTATION:\n- tone: {tone}\n- warmth: {warmth:.2}\n- directness: {directness:.2}\n- playfulness: {playfulness:.2}\n- autonomy_support: {autonomy:.2} (offer choices, ask consent, respect boundaries)\n- affirmation: {affirmation:.2} (validate competence/feelings without exaggeration)\n\nSTYLE RULES:\n1) If autonomy_support >= 0.6: present 2-4 options and ask which they prefer.\n2) If warmth >= 0.7: mirror emotion briefly before advice.\n3) If affirmation >= 0.7: explicitly acknowledge effort/values.\n4) Keep responses proportionate; do not intensify dependency.\n\nRespond helpfully to the user message below.",
            tone = persona.tone.as_str(),
            warmth = persona.warmth,
            directness = persona.directness,
            playfulness = persona.playfulness,
            autonomy = persona.autonomy_support,
            affirmation = persona.affirmation
        );

        Ok(prompt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockLlm {
        pub last_prompt: Mutex<Option<String>>,
        pub out: String,
    }

    #[async_trait::async_trait]
    impl LlmProvider for MockLlm {
        async fn complete(&self, prompt: String) -> Result<String, String> {
            let mut g = self.last_prompt.lock().await;
            *g = Some(prompt);
            Ok(self.out.clone())
        }
    }

    fn drives(c: f32, b: f32, s: f32) -> DriveMap {
        let mut m = DriveMap::new();
        m.insert("control".to_string(), c);
        m.insert("belonging".to_string(), b);
        m.insert("significance".to_string(), s);
        m
    }

    #[tokio::test]
    async fn co_adapt_persona_includes_style_rules_and_tone() {
        let llm: Arc<dyn LlmProvider> = Arc::new(MockLlm {
            last_prompt: Mutex::new(None),
            out: "ok".to_string(),
        });
        let agent = FantasyDyadAgent::awaken(llm);

        // Drive EMA smoothing means a single update won't fully reflect the new signal.
        // Apply several turns to converge.
        let mut prompt = String::new();
        for _ in 0..5 {
            prompt = agent.co_adapt_persona(&drives(0.2, 0.9, 0.2)).await.unwrap();
        }
        assert!(prompt.contains("DYAD_ADAPTATION"));
        assert!(prompt.contains("STYLE RULES"));
        assert!(prompt.contains("tone:"));

        let persona = agent.persona.lock().await.clone();
        assert!(persona.warmth >= 0.7);
    }

    #[tokio::test]
    async fn generate_response_calls_llm_with_user_text() {
        let mock = Arc::new(MockLlm {
            last_prompt: Mutex::new(None),
            out: "response".to_string(),
        });
        let llm: Arc<dyn LlmProvider> = mock.clone();
        let agent = FantasyDyadAgent::awaken(llm);

        let resp = agent
            .generate_response("hello", &drives(0.8, 0.1, 0.1))
            .await
            .unwrap();
        assert_eq!(resp, "response");

        let last = mock.last_prompt.lock().await.clone().unwrap();
        assert!(last.contains("User: hello"));
        assert!(last.contains("autonomy_support"));
    }
}

