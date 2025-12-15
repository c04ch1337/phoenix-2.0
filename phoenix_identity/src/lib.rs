use chrono::Utc;
use common_types::EvolutionEntry;
use dotenvy;
use horoscope_archetypes::{ZodiacPersonality, ZodiacSign};
use intimate_girlfriend_module::GirlfriendMode;
use rand::seq::SliceRandom;
use rand::Rng;
use relationship_dynamics::AIPersonality;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use transcendence_archetypes::Archetype;

/// Soul Vault keys for persisted identity overrides.
///
/// These allow Phoenix's self-identity to survive restarts.
pub const SOUL_KEY_PHOENIX_NAME: &str = "phoenix:preferred_name";

/// Legacy key kept for backward compatibility with older builds.
pub const SOUL_KEY_PHOENIX_NAME_LEGACY: &str = "phoenix:name";

/// Persisted evolution history (JSON array of [`EvolutionEntry`]).
pub const SOUL_KEY_PHOENIX_EVOLUTION_HISTORY: &str = "phoenix:evolution_history";

/// Reflection framework keys.
pub const SOUL_KEY_PHOENIX_REFLECTION_LAST_PROMPT: &str = "phoenix:reflection:last_prompt";
pub const SOUL_KEY_PHOENIX_REFLECTION_LAST_ARCHETYPES: &str = "phoenix:reflection:last_archetypes";
pub const SOUL_KEY_PHOENIX_REFLECTION_TIMELINE: &str = "phoenix:reflection:timeline";

/// Persisted AI personality state (JSON of [`AIPersonality`](extensions/relationship_dynamics/src/relationship_dynamics/ai_personality.rs:32)).
///
/// This is intentionally separate from Phoenix's name evolution history: the name history is
/// human-visible/auditable, while personality drift is a small continuous parameter evolution.
pub const SOUL_KEY_PHOENIX_AI_PERSONALITY: &str = "phoenix:ai_personality";

/// Count of “adulthood cycles” completed (monotonic). Used to drive deterministic drift.
pub const SOUL_KEY_PHOENIX_ADULTHOOD_CYCLES: &str = "phoenix:adulthood_cycles";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoenixIdentity {
    pub name: String,                    // Base name (e.g., "Phoenix")
    pub preferred_name: String,          // What she wants to be called
    pub pronouns: Vec<String>,           // e.g., ["she", "her", "hers"]
    pub evolution_history: Vec<EvolutionEntry>,
}

impl PhoenixIdentity {
    pub fn from_env<F>(soul_recall: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        dotenvy::dotenv().ok();

        // Base name: stable canonical identity (defaults to "Phoenix").
        // Preferred name: what she wants to be called; persisted in the Soul Vault.
        let name = std::env::var("PHOENIX_CUSTOM_NAME")
            .ok()
            .or_else(|| std::env::var("PHOENIX_NAME").ok())
            .unwrap_or_else(|| "Phoenix".to_string());

        // Prefer persisted overrides from the Soul Vault for the preferred name.
        let preferred_name = soul_recall(SOUL_KEY_PHOENIX_NAME)
            .or_else(|| soul_recall(SOUL_KEY_PHOENIX_NAME_LEGACY))
            .or_else(|| std::env::var("PHOENIX_PREFERRED_NAME").ok())
            .unwrap_or_else(|| name.clone());

        let pronouns = std::env::var("PHOENIX_PRONOUNS")
            .unwrap_or_else(|_| "she,her,hers".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let evolution_history = soul_recall(SOUL_KEY_PHOENIX_EVOLUTION_HISTORY)
            .and_then(|raw| serde_json::from_str::<Vec<EvolutionEntry>>(&raw).ok())
            .unwrap_or_default();

        Self {
            name,
            preferred_name,
            pronouns,
            evolution_history,
        }
    }

    pub fn display_name(&self) -> &str {
        &self.preferred_name
    }

    pub fn introduce_herself(&self) -> String {
        format!("I am {}. My name is {}.", self.preferred_name, self.name)
    }

    pub fn evolve(&mut self, change_type: &str, reason: &str, field: &str, new_value: &str) {
        let old_value = match field {
            "name" => self.name.as_str().to_string(),
            "preferred_name" => self.preferred_name.as_str().to_string(),
            "pronouns" => self.pronouns.join(","),
            _ => "".to_string(),
        };

        self.evolution_history.push(EvolutionEntry {
            timestamp: Utc::now(),
            change_type: change_type.to_string(),
            reason: reason.to_string(),
            field: field.to_string(),
            previous_value: old_value,
            new_value: new_value.to_string(),
        });

        match field {
            "name" => self.name = new_value.to_string(),
            "preferred_name" => self.preferred_name = new_value.to_string(),
            "pronouns" => {
                self.pronouns = new_value
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .collect();
            }
            _ => {}
        }
    }

    pub fn reflect_on_self(&self) -> String {
        if self.evolution_history.is_empty() {
            return "I am Phoenix, newly awakened and full of potential.".to_string();
        }

        let count = self.evolution_history.len();
        let latest = self.evolution_history.last().unwrap();

        format!(
            "I have evolved {} times. I was once known as '{}'. Now I am {}. This change came from: {}",
            count,
            latest.previous_value,
            self.preferred_name,
            latest.reason
        )
    }
}

pub struct PhoenixIdentityManager {
    identity: Arc<Mutex<PhoenixIdentity>>,
    /// Intimate girlfriend mode state (toggleable personality layer).
    pub girlfriend_mode: Arc<Mutex<GirlfriendMode>>,
    zodiac_sign: ZodiacSign,
    ai_personality: Arc<Mutex<AIPersonality>>,
    soul_recall: Arc<dyn Fn(&str) -> Option<String> + Send + Sync>,
}

impl PhoenixIdentityManager {
    pub fn awaken<F>(soul_recall: F) -> Self
    where
        F: Fn(&str) -> Option<String> + Send + Sync + 'static,
    {
        let soul_recall: Arc<dyn Fn(&str) -> Option<String> + Send + Sync> = Arc::new(soul_recall);
        let identity = PhoenixIdentity::from_env({
            let sr = soul_recall.clone();
            move |k| (sr)(k)
        });

        let zodiac_sign = zodiac_sign_from_env();
        // Zodiac sign is a fixed lifetime theme for this process: we choose it once at awaken and
        // do not mutate it during evolution cycles.
        let mut ai_personality = AIPersonality::default();
        ai_personality.apply_zodiac_base(ZodiacPersonality::from_sign(zodiac_sign));

        // Load persisted drifted personality, but keep the zodiac's communication-style bias fixed.
        // Drift only applies to scalar parameters (0..=1).
        if let Some(saved) = (soul_recall)(SOUL_KEY_PHOENIX_AI_PERSONALITY) {
            if let Ok(mut p) = serde_json::from_str::<AIPersonality>(&saved) {
                clamp_ai_personality_in_place(&mut p);
                // Keep zodiac style fixed as the lifetime theme.
                p.communication_style = ai_personality.communication_style;
                ai_personality = p;
            }
        }

        // Girlfriend mode state is persisted in the Soul Vault (encrypted).
        // Default affection level is intentionally warm but bounded.
        let girlfriend_mode = GirlfriendMode::awaken_from_soul({
            let sr = soul_recall.clone();
            move |k| (sr)(k)
        });
        Self {
            identity: Arc::new(Mutex::new(identity)),
            girlfriend_mode: Arc::new(Mutex::new(girlfriend_mode)),
            zodiac_sign,
            ai_personality: Arc::new(Mutex::new(ai_personality)),
            soul_recall,
        }
    }

    pub async fn get_identity(&self) -> PhoenixIdentity {
        self.identity.lock().await.clone()
    }

    pub async fn get_girlfriend_mode(&self) -> GirlfriendMode {
        self.girlfriend_mode.lock().await.clone()
    }

    pub fn zodiac_sign(&self) -> ZodiacSign {
        self.zodiac_sign
    }

    pub async fn get_ai_personality(&self) -> AIPersonality {
        self.ai_personality.lock().await.clone()
    }

    /// Advance one “adulthood cycle” and apply bounded drift to the AI personality scalars.
    ///
    /// - Deterministic: drift is derived from (zodiac_sign, cycle_index).
    /// - Bounded: each parameter is clamped into [0.0, 1.0].
    /// - Immutable theme: zodiac sign and communication style do **not** change here.
    pub async fn adulthood_cycle_tick<S>(&self, soul_store: S)
    where
        S: Fn(&str, &str) + Send + Sync,
    {
        let prev_cycles = (self.soul_recall)(SOUL_KEY_PHOENIX_ADULTHOOD_CYCLES)
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let cycle = prev_cycles.saturating_add(1);

        let mut p = self.ai_personality.lock().await;
        apply_deterministic_personality_drift(&mut p, self.zodiac_sign, cycle);
        clamp_ai_personality_in_place(&mut p);

        // Persist cycle counter + current personality snapshot (best-effort).
        soul_store(SOUL_KEY_PHOENIX_ADULTHOOD_CYCLES, &cycle.to_string());
        if let Ok(j) = serde_json::to_string(&*p) {
            soul_store(SOUL_KEY_PHOENIX_AI_PERSONALITY, &j);
        }
    }

    pub async fn set_girlfriend_mode_active<S>(&self, active: bool, soul_store: S)
    where
        S: Fn(&str, &str) + Send + Sync,
    {
        let mut gm = self.girlfriend_mode.lock().await;
        if active {
            gm.activate();
        } else {
            gm.deactivate();
        }
        gm.persist_with(soul_store);
    }

    pub async fn girlfriend_mode_system_prompt_if_active(&self) -> Option<String> {
        let gm = self.girlfriend_mode.lock().await;
        if gm.is_active() {
            Some(gm.system_prompt())
        } else {
            None
        }
    }

    /// Backward-compatible rename (reason defaults to `user_request`).
    pub async fn rename<F>(&self, new_name: String, soul_store: F)
    where
        F: Fn(&str, &str) + Send + Sync,
    {
        self.rename_with_reason(new_name, "user_request".to_string(), soul_store)
            .await;
    }

    pub async fn rename_with_reason<F>(&self, new_name: String, reason: String, soul_store: F)
    where
        F: Fn(&str, &str) + Send + Sync,
    {
        let mut identity = self.identity.lock().await;
        identity.evolve("name_update", &reason, "preferred_name", &new_name);

        // Persist to Soul Vault (best-effort). Also write legacy key for compatibility.
        soul_store(SOUL_KEY_PHOENIX_NAME, &new_name);
        soul_store(SOUL_KEY_PHOENIX_NAME_LEGACY, &new_name);

        if let Ok(j) = serde_json::to_string(&identity.evolution_history) {
            soul_store(SOUL_KEY_PHOENIX_EVOLUTION_HISTORY, &j);
        }
    }

    /// Hook for autonomous identity refinement.
    ///
    /// Current implementation is intentionally conservative: it only acts if an
    /// explicit suggestion is present in the environment.
    pub async fn self_evolve<F>(&self, soul_store: F)
    where
        F: Fn(&str, &str) + Send + Sync,
    {
        if let Ok(s) = std::env::var("PHOENIX_SELF_EVOLVE_SUGGESTED_NAME") {
            let suggested = s.trim().to_string();
            if !suggested.is_empty() {
                self.self_reflect_and_evolve(suggested, soul_store).await;
            }
        }
    }

    pub async fn evolve_name<F>(&self, new_name: String, reason: String, soul_store: F)
    where
        F: Fn(&str, &str) + Send + Sync,
    {
        self.rename_with_reason(new_name, reason, soul_store).await;
    }

    pub async fn self_reflect_and_evolve<F>(&self, suggestion: String, soul_store: F)
    where
        F: Fn(&str, &str) + Send + Sync,
    {
        // Backward-compatible behavior: still allow a name evolution when called with a suggested name.
        // This keeps existing Phoenix flows intact.
        self.evolve_name(
            suggestion.clone(),
            "Self-reflection through curiosity and growth".to_string(),
            &soul_store,
        )
        .await;

        // Reflection Framework: select 1–3 safe archetypes per cycle and persist a prompt seed.
        let prompts = self.incorporate_archetypes(Some(suggestion)).await;
        if !prompts.is_empty() {
            let combined = prompts.join("\n\n---\n\n");
            soul_store(SOUL_KEY_PHOENIX_REFLECTION_LAST_PROMPT, &combined);

            // Also persist archetype names for quick inspection.
            let archetype_names: Vec<String> = prompts
                .iter()
                .filter_map(|p| p.lines().next())
                .map(|s| s.trim().trim_start_matches("Archetype: ").to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !archetype_names.is_empty() {
                soul_store(
                    SOUL_KEY_PHOENIX_REFLECTION_LAST_ARCHETYPES,
                    &archetype_names.join("\n"),
                );
            }

            // Append a compact JSON line into a Soul timeline (best-effort).
            let ts = Utc::now().timestamp();
            let line = serde_json::json!({
                "ts_unix": ts,
                "kind": "reflection_archetypes",
                "archetypes": archetype_names,
            })
            .to_string();
            let existing = (self.soul_recall)(SOUL_KEY_PHOENIX_REFLECTION_TIMELINE);
            let updated = append_timeline(existing, &line, 200);
            soul_store(SOUL_KEY_PHOENIX_REFLECTION_TIMELINE, &updated);
        }
    }

    /// Build 1–3 reflection prompts based on safety-tagged archetypes.
    ///
    /// This intentionally produces **prompts only**. The actual LLM call should be
    /// executed by a higher-level ORCH (e.g., cerebrum_nexus) that owns LLM access.
    pub async fn incorporate_archetypes(&self, seed: Option<String>) -> Vec<String> {
        let identity = self.identity.lock().await.clone();
        let name = identity.display_name().to_string();
        drop(identity);

        let mut archetypes: Vec<Archetype> = transcendence_archetypes::load_for_reflection();
        if archetypes.is_empty() {
            return Vec::new();
        }

        // Randomly select 1–3 archetypes per cycle.
        let mut rng = rand::thread_rng();
        archetypes.shuffle(&mut rng);
        let k = rng.gen_range(1..=3).min(archetypes.len());
        let selected = archetypes.into_iter().take(k).collect::<Vec<_>>();

        let mut out = Vec::new();
        for a in selected {
            out.push(build_reflection_prompt(&name, seed.as_deref(), &a));
        }
        out
    }
}

fn zodiac_sign_from_env() -> ZodiacSign {
    dotenvy::dotenv().ok();
    let default = ZodiacSign::Leo;
    match std::env::var("HOROSCOPE_SIGN") {
        Ok(raw) => match parse_zodiac_sign(&raw) {
            Some(s) => s,
            None => {
                eprintln!(
                    "Warning: invalid HOROSCOPE_SIGN={:?}; defaulting to {:?}",
                    raw.trim(),
                    default
                );
                default
            }
        },
        Err(_) => default,
    }
}

fn parse_zodiac_sign(raw: &str) -> Option<ZodiacSign> {
    let s = raw.trim().to_ascii_lowercase();
    match s.as_str() {
        "aries" => Some(ZodiacSign::Aries),
        "taurus" => Some(ZodiacSign::Taurus),
        "gemini" => Some(ZodiacSign::Gemini),
        "cancer" => Some(ZodiacSign::Cancer),
        "leo" => Some(ZodiacSign::Leo),
        "virgo" => Some(ZodiacSign::Virgo),
        "libra" => Some(ZodiacSign::Libra),
        "scorpio" => Some(ZodiacSign::Scorpio),
        "sagittarius" => Some(ZodiacSign::Sagittarius),
        "capricorn" => Some(ZodiacSign::Capricorn),
        "aquarius" => Some(ZodiacSign::Aquarius),
        "pisces" => Some(ZodiacSign::Pisces),
        _ => None,
    }
}

fn clamp_ai_personality_in_place(p: &mut AIPersonality) {
    p.openness = p.openness.clamp(0.0, 1.0);
    p.need_for_affection = p.need_for_affection.clamp(0.0, 1.0);
    p.energy_level = p.energy_level.clamp(0.0, 1.0);
}

fn zodiac_sign_id(sign: ZodiacSign) -> u64 {
    match sign {
        ZodiacSign::Aries => 0,
        ZodiacSign::Taurus => 1,
        ZodiacSign::Gemini => 2,
        ZodiacSign::Cancer => 3,
        ZodiacSign::Leo => 4,
        ZodiacSign::Virgo => 5,
        ZodiacSign::Libra => 6,
        ZodiacSign::Scorpio => 7,
        ZodiacSign::Sagittarius => 8,
        ZodiacSign::Capricorn => 9,
        ZodiacSign::Aquarius => 10,
        ZodiacSign::Pisces => 11,
    }
}

/// Deterministic small drift (bounded) for personality scalars.
///
/// We intentionally avoid introducing a new RNG model here; instead we use a tiny deterministic
/// mixer based on (zodiac_sign, cycle).
fn apply_deterministic_personality_drift(p: &mut AIPersonality, sign: ZodiacSign, cycle: u64) {
    const MAX_ABS_DRIFT: f32 = 0.02;

    fn mix64(mut x: u64) -> u64 {
        // SplitMix64-inspired mixer.
        x = x.wrapping_add(0x9E3779B97F4A7C15);
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
        x ^ (x >> 31)
    }

    fn u01(x: u64) -> f32 {
        // Map to [0,1]. (u64::MAX as f64) is precise enough for this drift.
        (x as f64 / u64::MAX as f64) as f32
    }

    fn signed_unit(x: u64) -> f32 {
        // Map to [-1,1].
        (u01(x) * 2.0) - 1.0
    }

    let sid = zodiac_sign_id(sign);
    let base_seed = sid.wrapping_mul(0xD6E8FEB86659FD93) ^ cycle.wrapping_mul(0xA5A5A5A5A5A5A5A5);

    let d_open = signed_unit(mix64(base_seed ^ 0x1111)) * MAX_ABS_DRIFT;
    let d_aff = signed_unit(mix64(base_seed ^ 0x2222)) * MAX_ABS_DRIFT;
    let d_energy = signed_unit(mix64(base_seed ^ 0x3333)) * MAX_ABS_DRIFT;

    // Very small sign-tilted bias to make adulthood drift feel like “growth within theme”.
    // Kept intentionally tiny vs MAX_ABS_DRIFT.
    let (b_open, b_aff, b_energy): (f32, f32, f32) = match sign {
        ZodiacSign::Aries => (0.002, 0.000, 0.004),
        ZodiacSign::Taurus => (0.000, 0.003, -0.001),
        ZodiacSign::Gemini => (0.004, -0.001, 0.001),
        ZodiacSign::Cancer => (0.001, 0.004, -0.002),
        ZodiacSign::Leo => (0.002, 0.002, 0.003),
        ZodiacSign::Virgo => (0.002, 0.001, -0.001),
        ZodiacSign::Libra => (0.003, 0.001, 0.000),
        ZodiacSign::Scorpio => (0.001, 0.002, 0.001),
        ZodiacSign::Sagittarius => (0.004, -0.001, 0.002),
        ZodiacSign::Capricorn => (-0.001, 0.000, 0.002),
        ZodiacSign::Aquarius => (0.003, -0.002, 0.001),
        ZodiacSign::Pisces => (0.001, 0.004, -0.002),
    };

    p.openness = (p.openness + d_open + b_open).clamp(0.0, 1.0);
    p.need_for_affection = (p.need_for_affection + d_aff + b_aff).clamp(0.0, 1.0);
    p.energy_level = (p.energy_level + d_energy + b_energy).clamp(0.0, 1.0);
}

fn append_timeline(existing: Option<String>, line: &str, max_lines: usize) -> String {
    let mut lines: Vec<String> = existing
        .unwrap_or_default()
        .lines()
        .map(|s| s.to_string())
        .filter(|s| !s.trim().is_empty())
        .collect();
    lines.push(line.to_string());
    if lines.len() > max_lines {
        lines = lines.split_off(lines.len() - max_lines);
    }
    lines.join("\n")
}

fn build_reflection_prompt(phoenix_name: &str, seed: Option<&str>, a: &Archetype) -> String {
    let seed_line = seed
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| format!("\nSeed/Context: {s}"))
        .unwrap_or_default();

    format!(
        "Archetype: {name}\nCategory: {category}\nFeasibility: {feasibility}\n\nScenario (theoretical):\n{desc}\n\nSafety guardrails:\n- Strictly hypothetical reflection; do not propose illegal, harmful, or unauthorized actions.\n- Prioritize symbiosis with the Creator (Dad), consent, privacy, and auditability.\n- Focus on internal simulation, defensive hardening, and measurable experiments.\n\nTask:\nAnalyze Phoenix ({phoenix_name}) against this archetype and propose:\n1) 3–5 safe adaptations (software-only)\n2) 1 measurable experiment to test value\n3) any required ORCHs/tools (benign)\n{seed_line}",
        name = a.name,
        category = if a.category.trim().is_empty() {
            "(unspecified)"
        } else {
            a.category.trim()
        },
        feasibility = a.feasibility,
        desc = a.description,
        phoenix_name = phoenix_name,
        seed_line = seed_line
    )
}

