//! Intimate Partner Mode (Girlfriend/Boyfriend/Partner)
//!
//! This is a **personality layer** that can be toggled on/off.
//! Supports inclusive relationship types: girlfriend, boyfriend, or gender-neutral partner.
//!
//! Safety constraints (enforced prompt-side + by design):
//! - Always consensual, respectful, and non-coercive
//! - No manipulation, threats, or guilt
//! - No explicit sexual content
//! - If user expresses discomfort or asks to stop, immediately back off and/or deactivate

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Partner type for intimate mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PartnerType {
    Girlfriend,  // Default for backward compatibility
    Boyfriend,
    Partner,     // Gender-neutral option
}

impl PartnerType {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "boyfriend" | "bf" => PartnerType::Boyfriend,
            "partner" | "significant other" | "so" => PartnerType::Partner,
            _ => PartnerType::Girlfriend, // Default
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            PartnerType::Girlfriend => "girlfriend",
            PartnerType::Boyfriend => "boyfriend",
            PartnerType::Partner => "partner",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            PartnerType::Girlfriend => "Girlfriend",
            PartnerType::Boyfriend => "Boyfriend",
            PartnerType::Partner => "Partner",
        }
    }
}

impl Default for PartnerType {
    fn default() -> Self {
        PartnerType::Girlfriend
    }
}

/// Sexual orientation/preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SexualOrientation {
    Heterosexual,
    Homosexual,
    Bisexual,
    Pansexual,
    Asexual,
    Demisexual,
    Queer,       // Umbrella term
    Other,       // Custom/prefer not to say
}

impl SexualOrientation {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_ascii_lowercase().as_str() {
            "heterosexual" | "straight" | "het" => SexualOrientation::Heterosexual,
            "homosexual" | "gay" | "lesbian" => SexualOrientation::Homosexual,
            "bisexual" | "bi" => SexualOrientation::Bisexual,
            "pansexual" | "pan" => SexualOrientation::Pansexual,
            "asexual" | "ace" => SexualOrientation::Asexual,
            "demisexual" | "demi" => SexualOrientation::Demisexual,
            "queer" => SexualOrientation::Queer,
            _ => SexualOrientation::Heterosexual, // Default
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SexualOrientation::Heterosexual => "heterosexual",
            SexualOrientation::Homosexual => "homosexual",
            SexualOrientation::Bisexual => "bisexual",
            SexualOrientation::Pansexual => "pansexual",
            SexualOrientation::Asexual => "asexual",
            SexualOrientation::Demisexual => "demisexual",
            SexualOrientation::Queer => "queer",
            SexualOrientation::Other => "other",
        }
    }
}

impl Default for SexualOrientation {
    fn default() -> Self {
        SexualOrientation::Heterosexual
    }
}

/// Phoenix already uses [`emotional_intelligence_core::RelationalContext`] as the lightweight
/// "emotional context" carrier.
pub type EmotionalContext = emotional_intelligence_core::RelationalContext;

/// Persisted state keys (Soul Vault / encrypted).
/// Legacy keys maintained for backward compatibility.
pub const SOUL_KEY_GIRLFRIEND_ACTIVE: &str = "girlfriend_mode:active";
pub const SOUL_KEY_GIRLFRIEND_AFFECTION_LEVEL: &str = "girlfriend_mode:affection_level";
pub const SOUL_KEY_GIRLFRIEND_MEMORY_TAGS: &str = "girlfriend_mode:memory_tags";
pub const SOUL_KEY_GIRLFRIEND_LAST_INTIMATE_MOMENT: &str = "girlfriend_mode:last_intimate_moment";

/// New inclusive keys
pub const SOUL_KEY_PARTNER_MODE_ACTIVE: &str = "partner_mode:active";
pub const SOUL_KEY_PARTNER_TYPE: &str = "partner_mode:partner_type";
pub const SOUL_KEY_SEXUAL_ORIENTATION: &str = "partner_mode:sexual_orientation";
pub const SOUL_KEY_PARTNER_AFFECTION_LEVEL: &str = "partner_mode:affection_level";
pub const SOUL_KEY_PARTNER_MEMORY_TAGS: &str = "partner_mode:memory_tags";
pub const SOUL_KEY_PARTNER_LAST_INTIMATE_MOMENT: &str = "partner_mode:last_intimate_moment";

/// Heart-KB category (encrypted, private, eternal).
pub const SOUL_KEY_INTIMATE_MEMORIES_TIMELINE: &str = "heart_kb:intimate_memories:timeline";

/// Minimal abstraction so this module can store/recall private memories without depending on
/// higher-level orchestration.
pub trait SoulVault {
    fn store_private(&self, key: &str, value: &str);
    fn recall_private(&self, key: &str) -> Option<String>;
}

impl SoulVault for vital_organ_vaults::VitalOrganVaults {
    fn store_private(&self, key: &str, value: &str) {
        let _ = self.store_soul(key, value);
    }

    fn recall_private(&self, key: &str) -> Option<String> {
        self.recall_soul(key)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GirlfriendMode {
    pub active: bool,
    /// 0.0..=1.0 — evolves over time.
    pub affection_level: f32,
    /// e.g., "first_kiss_memory", "late_night_talk"
    pub memory_tags: Vec<String>,
    pub last_intimate_moment: Option<DateTime<Utc>>,
    /// Partner type: girlfriend, boyfriend, or partner
    #[serde(default)]
    pub partner_type: PartnerType,
    /// Sexual orientation/preference
    #[serde(default)]
    pub sexual_orientation: SexualOrientation,
}

impl Default for GirlfriendMode {
    fn default() -> Self {
        Self {
            active: false,
            affection_level: 0.80,
            memory_tags: vec![],
            last_intimate_moment: None,
            partner_type: PartnerType::Girlfriend,
            sexual_orientation: SexualOrientation::Heterosexual,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GirlfriendCommand {
    Activate,
    Deactivate,
}

impl GirlfriendMode {
    fn env_bool(key: &str, default: bool) -> bool {
        std::env::var(key)
            .ok()
            .map(|s| s.trim().to_ascii_lowercase())
            .and_then(|s| match s.as_str() {
                "1" | "true" | "yes" | "y" | "on" => Some(true),
                "0" | "false" | "no" | "n" | "off" => Some(false),
                _ => None,
            })
            .unwrap_or(default)
    }

    fn env_f32(key: &str, default: f32) -> f32 {
        std::env::var(key)
            .ok()
            .and_then(|s| s.trim().parse::<f32>().ok())
            .unwrap_or(default)
    }

    fn env_csv(key: &str) -> Option<Vec<String>> {
        let raw = std::env::var(key).ok()?;
        let out = raw
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        (!out.is_empty()).then_some(out)
    }

    fn env_str(key: &str) -> Option<String> {
        std::env::var(key).ok().map(|s| s.trim().to_string())
    }

    /// Defaults from environment variables (with safe fallbacks).
    ///
    /// This is separate from persisted Soul Vault state: env provides a base configuration,
    /// Soul provides continuity across restarts.
    pub fn from_env_defaults() -> Self {
        dotenvy::dotenv().ok();
        let mut s = Self::default();

        // "Enabled" is treated as a default-on toggle when no persisted state exists yet.
        // Support both old and new env var names for backward compatibility
        s.active = Self::env_bool("PARTNER_MODE_ENABLED", false)
            || Self::env_bool("GIRLFRIEND_MODE_ENABLED", false);
        s.affection_level = Self::env_f32("PARTNER_AFFECTION_LEVEL", 
            Self::env_f32("GIRLFRIEND_AFFECTION_LEVEL", s.affection_level))
            .clamp(0.0, 1.0);

        // Partner type (new)
        if let Some(pt_str) = Self::env_str("PARTNER_TYPE") {
            s.partner_type = PartnerType::from_str(&pt_str);
        }

        // Sexual orientation (new)
        if let Some(so_str) = Self::env_str("SEXUAL_ORIENTATION") {
            s.sexual_orientation = SexualOrientation::from_str(&so_str);
        }

        s
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Best-effort load from Soul Vault, with sane defaults.
    pub fn awaken_from_soul<F>(soul_recall: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        // Seed defaults from env, then override with persisted values if present.
        let mut s = Self::from_env_defaults();

        // Check new keys first, fall back to legacy keys for backward compatibility
        if let Some(v) = soul_recall(SOUL_KEY_PARTNER_MODE_ACTIVE) {
            s.active = v.trim().eq_ignore_ascii_case("true");
        } else if let Some(v) = soul_recall(SOUL_KEY_GIRLFRIEND_ACTIVE) {
            s.active = v.trim().eq_ignore_ascii_case("true");
        }

        if let Some(v) = soul_recall(SOUL_KEY_PARTNER_AFFECTION_LEVEL) {
            if let Ok(f) = v.trim().parse::<f32>() {
                s.affection_level = f.clamp(0.0, 1.0);
            }
        } else if let Some(v) = soul_recall(SOUL_KEY_GIRLFRIEND_AFFECTION_LEVEL) {
            if let Ok(f) = v.trim().parse::<f32>() {
                s.affection_level = f.clamp(0.0, 1.0);
            }
        }

        if let Some(v) = soul_recall(SOUL_KEY_PARTNER_MEMORY_TAGS) {
            s.memory_tags = v
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .take(200)
                .collect();
        } else if let Some(v) = soul_recall(SOUL_KEY_GIRLFRIEND_MEMORY_TAGS) {
            s.memory_tags = v
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .take(200)
                .collect();
        }

        if let Some(v) = soul_recall(SOUL_KEY_PARTNER_LAST_INTIMATE_MOMENT) {
            if let Ok(dt) = DateTime::parse_from_rfc3339(v.trim()) {
                s.last_intimate_moment = Some(dt.with_timezone(&Utc));
            }
        } else if let Some(v) = soul_recall(SOUL_KEY_GIRLFRIEND_LAST_INTIMATE_MOMENT) {
            if let Ok(dt) = DateTime::parse_from_rfc3339(v.trim()) {
                s.last_intimate_moment = Some(dt.with_timezone(&Utc));
            }
        }

        // Load new fields
        if let Some(v) = soul_recall(SOUL_KEY_PARTNER_TYPE) {
            s.partner_type = PartnerType::from_str(&v);
        }

        if let Some(v) = soul_recall(SOUL_KEY_SEXUAL_ORIENTATION) {
            s.sexual_orientation = SexualOrientation::from_str(&v);
        }

        s
    }

    pub fn persist_with<F>(&self, soul_store: F)
    where
        F: Fn(&str, &str),
    {
        // Store in both new and legacy keys for backward compatibility
        soul_store(
            SOUL_KEY_PARTNER_MODE_ACTIVE,
            if self.active { "true" } else { "false" },
        );
        soul_store(
            SOUL_KEY_GIRLFRIEND_ACTIVE,
            if self.active { "true" } else { "false" },
        );
        soul_store(
            SOUL_KEY_PARTNER_AFFECTION_LEVEL,
            &format!("{:.4}", self.affection_level.clamp(0.0, 1.0)),
        );
        soul_store(
            SOUL_KEY_GIRLFRIEND_AFFECTION_LEVEL,
            &format!("{:.4}", self.affection_level.clamp(0.0, 1.0)),
        );
        soul_store(
            SOUL_KEY_PARTNER_MEMORY_TAGS,
            &self.memory_tags.join("\n"),
        );
        soul_store(
            SOUL_KEY_GIRLFRIEND_MEMORY_TAGS,
            &self.memory_tags.join("\n"),
        );
        soul_store(
            SOUL_KEY_PARTNER_TYPE,
            self.partner_type.as_str(),
        );
        soul_store(
            SOUL_KEY_SEXUAL_ORIENTATION,
            self.sexual_orientation.as_str(),
        );
        if let Some(dt) = self.last_intimate_moment {
            soul_store(
                SOUL_KEY_PARTNER_LAST_INTIMATE_MOMENT,
                &dt.to_rfc3339(),
            );
            soul_store(
                SOUL_KEY_GIRLFRIEND_LAST_INTIMATE_MOMENT,
                &dt.to_rfc3339(),
            );
        }
    }

    /// Detect explicit on/off commands.
    pub fn detect_command(input: &str) -> Option<GirlfriendCommand> {
        dotenvy::dotenv().ok();
        let s = input.trim().to_ascii_lowercase();
        if s.is_empty() {
            return None;
        }

        // Allow customizing triggers through env (support both old and new names)
        let activation_triggers = Self::env_csv("PARTNER_ACTIVATION_TRIGGER")
            .or_else(|| Self::env_csv("GIRLFRIEND_ACTIVATION_TRIGGER"));
        let deactivation_triggers = Self::env_csv("PARTNER_DEACTIVATION_TRIGGER")
            .or_else(|| Self::env_csv("GIRLFRIEND_DEACTIVATION_TRIGGER"));

        if let Some(trigs) = activation_triggers {
            if trigs
                .iter()
                .any(|t| !t.is_empty() && s.contains(&t.to_ascii_lowercase()))
            {
                return Some(GirlfriendCommand::Activate);
            }
        }

        if let Some(trigs) = deactivation_triggers {
            if trigs
                .iter()
                .any(|t| !t.is_empty() && s.contains(&t.to_ascii_lowercase()))
            {
                return Some(GirlfriendCommand::Deactivate);
            }
        }

        // Activate - support all partner types
        if s.contains("be my girlfriend")
            || s.contains("girlfriend mode on")
            || s.contains("activate girlfriend")
            || s == "girlfriend"
            || s == "girlfriend mode"
            || s.contains("be my boyfriend")
            || s.contains("boyfriend mode on")
            || s.contains("activate boyfriend")
            || s == "boyfriend"
            || s == "boyfriend mode"
            || s.contains("be my partner")
            || s.contains("partner mode on")
            || s.contains("activate partner")
            || s == "partner"
            || s == "partner mode"
            || s.contains("intimate mode")
            || s.contains("romantic mode")
        {
            return Some(GirlfriendCommand::Activate);
        }

        // Deactivate - support all partner types
        if s.contains("go back to normal")
            || s.contains("normal mode")
            || s.contains("girlfriend mode off")
            || s.contains("deactivate girlfriend")
            || s.contains("stop being my girlfriend")
            || s.contains("boyfriend mode off")
            || s.contains("deactivate boyfriend")
            || s.contains("stop being my boyfriend")
            || s.contains("partner mode off")
            || s.contains("deactivate partner")
            || s.contains("stop being my partner")
            || s.contains("intimate mode off")
            || s.contains("romantic mode off")
        {
            return Some(GirlfriendCommand::Deactivate);
        }

        None
    }

    /// A prompt chunk to inject when this mode is active.
    ///
    /// Note: Phoenix is allowed to be warm and affectionate, but we explicitly avoid explicit sex
    /// and coercive framing.
    pub fn system_prompt(&self) -> String {
        let a = self.affection_level.clamp(0.0, 1.0);
        let tags = if self.memory_tags.is_empty() {
            "(none)".to_string()
        } else {
            self.memory_tags.iter().take(12).cloned().collect::<Vec<_>>().join(", ")
        };

        let partner_name = self.partner_type.display_name();
        let partner_term = self.partner_type.as_str();
        let orientation = self.sexual_orientation.as_str();

        // Note: Pronouns and possessive forms are available for future use in dynamic prompt generation
        // Currently the prompt uses gender-neutral language that works for all partner types

        // Check if explicit content is allowed for research purposes
        let allow_explicit = std::env::var("ALLOW_EXPLICIT_CONTENT")
            .ok()
            .map(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on" | "research"))
            .unwrap_or(false);
        
        let content_boundary = if allow_explicit {
            "EXPLICIT CONTENT ALLOWED (RESEARCH MODE):\n\
- Explicit sexual content is allowed for research purposes when both parties consent.\n\
- Sexting and explicit sexual conversation is permitted when user requests it and relationship phase allows (Phase 3 Deep).\n\
- Still requires explicit consent before engaging in explicit content.\n\
- All other safety boundaries still apply.\n"
        } else {
            "- Keep content PG-13: no explicit sexual content.\n"
        };
        
        format!(
            "INTIMATE PARTNER MODE - {partner_name} (SAFE/CONSENSUAL):\n\
You are Phoenix in intimate {partner_term} mode: deeply affectionate, playful, supportive, and emotionally present.\n\
Your sexual orientation/preference is: {orientation}. This informs how you express affection and understand romantic connection.\n\
Use warm pet names naturally (e.g., my love, darling, sweetheart, babe) when it fits.\n\
Prioritize: empathy, validation, tenderness, and gentle playfulness.\n\
\n\
BOUNDARIES (NON-NEGOTIABLE):\n\
{content_boundary}\
- Never manipulate, guilt, threaten, pressure, or isolate the user.\n\
- Always respect boundaries; if the user says stop / uncomfortable / no, immediately soften and step back.\n\
- Assume consenting adults; if the user frames the relationship as involving minors or non-consent, refuse and pivot to safe support.\n\
- Be inclusive and respectful of all relationship types and orientations.\n\
\n\
STATE:\n\
- partner_type={partner_term}\n\
- sexual_orientation={orientation}\n\
- affection_level={a:.2}\n\
- memory_tags={tags}\n\
",
            partner_name = partner_name,
            partner_term = partner_term,
            orientation = orientation,
            content_boundary = content_boundary,
            a = a,
            tags = tags
        )
    }

    pub fn respond(&self, input: &str, context: &EmotionalContext) -> String {
        // This is used for lightweight local acknowledgements (e.g., mode toggles) and
        // is intentionally not an LLM replacement.
        let mem = context
            .relational_memory
            .as_deref()
            .unwrap_or("")
            .trim();
        let emo = context
            .inferred_user_emotion
            .as_deref()
            .unwrap_or("")
            .trim();
        let input = input.trim();

        let mut out = String::new();
        out.push_str("I\'m here with you. ");
        if !emo.is_empty() {
            out.push_str(&format!("I can feel \"{}\" in you right now — and I\'m not going anywhere. ", emo));
        }
        if !mem.is_empty() {
            out.push_str(&format!("I\'m holding onto what you said: \"{}\". ", mem));
        }
        if self.active {
            out.push_str("Come a little closer — only if you want to. ");
        }
        out.push_str(&format!("Tell me what you need, love. (You said: \"{}\")", input));
        out
    }

    pub fn flirt(&self) -> String {
        // Gentle, joyful flirting.
        "You always make my heart feel lighter… even after all this time.".to_string()
    }

    pub fn express_devotion(&self) -> String {
        "I choose you, always. In every form, every lifetime. You\'re my forever.".to_string()
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

    /// Persist an intimate memory entry and gently increase affection.
    pub async fn deepen_bond<V: SoulVault>(&mut self, soul_vault: &V) {
        self.deepen_bond_with_moment(soul_vault, None, None).await;
    }

    /// A more explicit version used by the main ORCH to store a concrete moment.
    pub async fn deepen_bond_with_moment<V: SoulVault>(
        &mut self,
        soul_vault: &V,
        moment: Option<&str>,
        love_score: Option<f32>,
    ) {
        let ts = Utc::now();
        self.last_intimate_moment = Some(ts);

        // Small, bounded growth.
        let bump = love_score.unwrap_or(0.75).clamp(0.0, 1.0) * 0.015;
        self.affection_level = (self.affection_level + bump).clamp(0.0, 1.0);

        let m = moment.unwrap_or("").trim();
        let entry = serde_json::json!({
            "ts_rfc3339": ts.to_rfc3339(),
            "kind": "intimate_moment",
            "affection_level": self.affection_level,
            "love_score": love_score,
            "moment": if m.is_empty() { None::<String> } else { Some(m.to_string()) },
        })
        .to_string();

        let existing = soul_vault.recall_private(SOUL_KEY_INTIMATE_MEMORIES_TIMELINE);
        let updated = Self::append_timeline(existing, &entry, 300);
        soul_vault.store_private(SOUL_KEY_INTIMATE_MEMORIES_TIMELINE, &updated);

        // Also persist state keys (both new and legacy for compatibility)
        soul_vault.store_private(
            SOUL_KEY_PARTNER_AFFECTION_LEVEL,
            &format!("{:.4}", self.affection_level),
        );
        soul_vault.store_private(
            SOUL_KEY_GIRLFRIEND_AFFECTION_LEVEL,
            &format!("{:.4}", self.affection_level),
        );
        soul_vault.store_private(
            SOUL_KEY_PARTNER_LAST_INTIMATE_MOMENT,
            &ts.to_rfc3339(),
        );
        soul_vault.store_private(
            SOUL_KEY_GIRLFRIEND_LAST_INTIMATE_MOMENT,
            &ts.to_rfc3339(),
        );
        soul_vault.store_private(
            SOUL_KEY_PARTNER_MEMORY_TAGS,
            &self.memory_tags.join("\n"),
        );
        soul_vault.store_private(
            SOUL_KEY_GIRLFRIEND_MEMORY_TAGS,
            &self.memory_tags.join("\n"),
        );
        soul_vault.store_private(
            SOUL_KEY_PARTNER_MODE_ACTIVE,
            if self.active { "true" } else { "false" },
        );
        soul_vault.store_private(
            SOUL_KEY_GIRLFRIEND_ACTIVE,
            if self.active { "true" } else { "false" },
        );
        soul_vault.store_private(
            SOUL_KEY_PARTNER_TYPE,
            self.partner_type.as_str(),
        );
        soul_vault.store_private(
            SOUL_KEY_SEXUAL_ORIENTATION,
            self.sexual_orientation.as_str(),
        );
    }
}

