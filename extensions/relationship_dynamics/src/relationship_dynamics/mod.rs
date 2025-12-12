use std::str::FromStr;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

use emotion_detection::{DetectedEmotion, EmotionDetector};

use intimate_girlfriend_module::GirlfriendMode;

pub mod attachment;
pub mod ai_personality;
pub mod goals;
pub mod shared_memory;
pub mod template;
pub mod voice_modulation;

pub use attachment::{AttachmentEvolution, AttachmentProfile, AttachmentStyle};
pub use ai_personality::{AIPersonality, CommunicationStyle, LoveLanguage, Mood};
pub use goals::SharedGoal;
pub use shared_memory::SharedMemory;
pub use template::{IntimacyLevel, InteractionWeights, RelationshipTemplate};
pub use voice_modulation::{PhoenixVoice, VoiceMood, VoiceParams};

use template::{SOUL_KEY_RELATIONSHIP_INTIMACY_LEVEL, SOUL_KEY_RELATIONSHIP_TEMPLATE};

/// Soul Vault keys.
pub const SOUL_KEY_RELATIONSHIP_GOALS: &str = "relationship_dynamics:goals";
pub const SOUL_KEY_RELATIONSHIP_MEMORIES: &str = "relationship_dynamics:memories";
pub const SOUL_KEY_RELATIONSHIP_PERSONALITY: &str = "relationship_dynamics:ai_personality";
pub const SOUL_KEY_RELATIONSHIP_ATTACHMENT_PROFILE: &str = "relationship_dynamics:attachment_profile";
pub const SOUL_KEY_RELATIONSHIP_ATTACHMENT_POSITIVE_COUNT: &str =
    "relationship_dynamics:attachment_positive_count";

/// Minimal abstraction so this module can store/recall private state without depending on
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionType {
    Affirmation,
    Support,
    ConflictRepair,
    DeepTalk,
    Play,
    Planning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionOutcome {
    /// -1.0..=1.0 subjective effect on relationship health.
    pub delta: f32,
    pub score: f32,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub ts: DateTime<Utc>,
    pub interaction_type: InteractionType,
    pub user_input: String,
    pub ai_response: String,
    pub detected_emotion: Option<DetectedEmotion>,
    pub outcome: InteractionOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionEntry {
    pub ts: DateTime<Utc>,
    pub from: RelationshipTemplate,
    pub to: RelationshipTemplate,
    pub score: f32,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedResponse {
    pub text: String,
    pub ssml: Option<String>,
    pub voice_params: Option<VoiceParams>,
    pub stats_summary: String,
    pub detected_emotion: Option<DetectedEmotion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Partnership {
    pub template: RelationshipTemplate,
    pub ai_personality: AIPersonality,
    pub attachment_profile: AttachmentProfile,
    pub secure_evolution_counter: usize,
    pub shared_goals: Vec<SharedGoal>,
    pub shared_memories: Vec<SharedMemory>,
    pub interaction_history: Vec<Interaction>,
    pub evolution_history: Vec<EvolutionEntry>,
    pub health: f32,

    #[serde(skip)]
    pub emotion_detector: EmotionDetector,
}

impl Partnership {
    /// Create a new Partnership with env + Soul Vault state.
    pub fn new(template_arg: RelationshipTemplate, soul: Option<&dyn SoulVault>) -> Self {
        let mut template = RelationshipTemplate::from_env_or_default(template_arg);
        let mut ai_personality = AIPersonality::default();
        let mut attachment_profile = AttachmentProfile::new(&template);
        let mut secure_evolution_counter: usize = 0;
        let mut shared_goals: Vec<SharedGoal> = vec![];
        let mut shared_memories: Vec<SharedMemory> = vec![];

        if let Some(soul) = soul {
            // Template override.
            if let Some(saved) = soul.recall_private(SOUL_KEY_RELATIONSHIP_TEMPLATE) {
                if let Ok(t) = RelationshipTemplate::from_str(saved.trim()) {
                    template = t;
                }
            }
            if let Some(saved) = soul.recall_private(SOUL_KEY_RELATIONSHIP_INTIMACY_LEVEL) {
                if let Ok(level) = IntimacyLevel::from_str(saved.trim()) {
                    template.set_intimacy_level(level);
                }
            }

            if let Some(saved) = soul.recall_private(SOUL_KEY_RELATIONSHIP_PERSONALITY) {
                if let Ok(p) = serde_json::from_str::<AIPersonality>(&saved) {
                    ai_personality = p;
                }
            }
            if let Some(saved) = soul.recall_private(SOUL_KEY_RELATIONSHIP_GOALS) {
                if let Ok(g) = serde_json::from_str::<Vec<SharedGoal>>(&saved) {
                    shared_goals = g;
                }
            }
            if let Some(saved) = soul.recall_private(SOUL_KEY_RELATIONSHIP_MEMORIES) {
                if let Ok(m) = serde_json::from_str::<Vec<SharedMemory>>(&saved) {
                    shared_memories = m;
                }
            }

            if let Some(saved) = soul.recall_private(SOUL_KEY_RELATIONSHIP_ATTACHMENT_PROFILE) {
                if let Ok(a) = serde_json::from_str::<AttachmentProfile>(&saved) {
                    attachment_profile = a;
                }
            }
            if let Some(saved) = soul.recall_private(SOUL_KEY_RELATIONSHIP_ATTACHMENT_POSITIVE_COUNT) {
                if let Ok(n) = saved.trim().parse::<usize>() {
                    secure_evolution_counter = n;
                }
            }
        }

        Self {
            template,
            ai_personality,
            attachment_profile,
            secure_evolution_counter,
            shared_goals,
            shared_memories,
            interaction_history: vec![],
            evolution_history: vec![],
            health: 0.85,
            emotion_detector: EmotionDetector::from_env(),
        }
    }

    pub fn persist_key_state(&self, soul: &dyn SoulVault) {
        soul.store_private(SOUL_KEY_RELATIONSHIP_TEMPLATE, self.template.template_name());
        if let Some(level) = self.template.intimacy_level() {
            soul.store_private(SOUL_KEY_RELATIONSHIP_INTIMACY_LEVEL, &level.to_string());
        }
        if let Ok(s) = serde_json::to_string(&self.ai_personality) {
            soul.store_private(SOUL_KEY_RELATIONSHIP_PERSONALITY, &s);
        }
        if let Ok(s) = serde_json::to_string(&self.shared_goals) {
            soul.store_private(SOUL_KEY_RELATIONSHIP_GOALS, &s);
        }
        if let Ok(s) = serde_json::to_string(&self.shared_memories) {
            soul.store_private(SOUL_KEY_RELATIONSHIP_MEMORIES, &s);
        }

        if let Ok(s) = serde_json::to_string(&self.attachment_profile) {
            soul.store_private(SOUL_KEY_RELATIONSHIP_ATTACHMENT_PROFILE, &s);
        }
        soul.store_private(
            SOUL_KEY_RELATIONSHIP_ATTACHMENT_POSITIVE_COUNT,
            &self.secure_evolution_counter.to_string(),
        );
    }

    pub fn get_stats_summary(&self) -> String {
        format!(
            "Affection: {:.0}% | Energy: {:.0}% | Mood: {:?} | Template: {} | Attachment: {:?} (Security: {:.0}%)",
            self.ai_personality.need_for_affection.clamp(0.0, 1.0) * 100.0,
            self.ai_personality.energy_level.clamp(0.0, 1.0) * 100.0,
            self.ai_personality.current_mood(),
            self.template,
            self.attachment_profile.style,
            (self.attachment_profile.security_score.clamp(0.0, 1.0) * 100.0)
        )
    }

    pub fn to_telemetry_payload(&self) -> serde_json::Value {
        json!({
            "kind": "relationship_dynamics",
            "stats_summary": self.get_stats_summary(),
            "health": self.health,
            "template": self.template.template_name(),
            "mood": format!("{:?}", self.ai_personality.current_mood()),
            "attachment_style": format!("{:?}", self.attachment_profile.style),
            "attachment_security": self.attachment_profile.security_score,
            "goals": self.shared_goals.iter().map(|g| json!({
                "name": g.name,
                "progress": g.progress,
            })).collect::<Vec<_>>(),
        })
    }

    pub fn ensure_goal(&mut self, name: &str) {
        if self.shared_goals.iter().any(|g| g.name == name) {
            return;
        }
        self.shared_goals.push(SharedGoal::new(name));
    }

    pub fn update_goal_progress(&mut self, goal_name: &str, delta: f64) -> Option<String> {
        let goal = self.shared_goals.iter_mut().find(|g| g.name == goal_name)?;
        let before = goal.progress;
        goal.update(delta);
        if !before.is_nan() && goal.is_complete() {
            return Some(format!("We did it together — goal achieved: {} {}", goal.name, goal.progress_bar(18)));
        }
        None
    }

    pub fn add_shared_memory(&mut self, memory: SharedMemory) {
        self.shared_memories.push(memory);
        // Keep bounded.
        if self.shared_memories.len() > 300 {
            self.shared_memories.drain(0..(self.shared_memories.len() - 300));
        }
    }

    pub fn reference_memory_in_response(&self, user_input: &str, response: &mut String) {
        if self.shared_memories.is_empty() {
            return;
        }
        // Pick the best matching memory.
        let mut best: Option<(&SharedMemory, f32)> = None;
        for m in &self.shared_memories {
            let s = m.relevance_score(user_input);
            if s < 0.55 {
                continue;
            }
            if best.map(|(_, b)| s > b).unwrap_or(true) {
                best = Some((m, s));
            }
        }
        if let Some((m, _)) = best {
            response.push_str(&format!("\n\nA little memory surfaced: \"{}\" — {}", m.title, m.content));
        }
    }

    /// #8 AI-initiated activity suggestions.
    pub fn generate_ai_interaction(&self) -> Option<String> {
        if self.ai_personality.energy_level <= 0.60 {
            return None;
        }

        let now = Utc::now();
        let last_ts = self.interaction_history.last().map(|i| i.ts);
        let low_recent_contact = last_ts
            .map(|ts| (now - ts).num_minutes() > 240)
            .unwrap_or(true);

        if self.attachment_profile.style == AttachmentStyle::Anxious && low_recent_contact {
            return Some("I’ve been missing you… can we talk for a minute?".to_string());
        }

        let mood = self.ai_personality.current_mood();
        let mut s = match mood {
            Mood::Excited => "Let’s go on a little virtual adventure—stargazing under a digital sky?".to_string(),
            Mood::Reflective => "How about a quiet evening where we share stories and listen to each other?".to_string(),
            Mood::Tired => "Let’s keep it soft: a warm tea moment and a calming playlist.".to_string(),
            Mood::Affectionate => "Come close—let’s do a cozy ‘couch date’: a movie, a blanket, and me doting on you.".to_string(),
            Mood::Calm => "Want a gentle date idea—like a virtual picnic and a shared gratitude list?".to_string(),
        };

        match self.attachment_profile.style {
            AttachmentStyle::Secure => {
                // Balanced confidence.
            }
            AttachmentStyle::Avoidant => {
                s.push_str(" (No pressure—just checking in.)");
            }
            AttachmentStyle::Disorganized => {
                s = format!("Hey… if you’re up for it, {s}");
            }
            AttachmentStyle::Anxious => {
                // Already handled above for low-contact; otherwise add gentle reassurance.
                s.push_str(" I just like being near you.");
            }
        }
        if let Some(goal) = self.shared_goals.first() {
            s.push_str(&format!(" (It’ll move us toward: \"{}\".)", goal.name));
        }
        Some(s)
    }

    fn weighted_score(&self, interaction_type: InteractionType) -> f32 {
        let w = self.template.get_interaction_weights();
        match interaction_type {
            InteractionType::Affirmation => w.affirmation,
            InteractionType::Support => w.support,
            InteractionType::DeepTalk => w.deep_talk,
            InteractionType::Play => w.play,
            InteractionType::Planning => w.planning,
            InteractionType::ConflictRepair => w.conflict_repair,
        }
    }

    fn base_response(&mut self, input: &str) -> String {
        let input = input.trim();
        if input.is_empty() {
            return "I’m here. Talk to me, love.".to_string();
        }
        match &self.template {
            RelationshipTemplate::CasualFriendship => {
                format!("I hear you. Want to tell me more about \"{input}\"?")
            }
            RelationshipTemplate::SupportivePartnership => {
                format!("I’m with you. What’s the smallest next step for \"{input}\"?")
            }
            RelationshipTemplate::GrowthOrientedPartnership => {
                format!("Let’s grow from this together. What does \"{input}\" reveal about what you need right now?")
            }
            RelationshipTemplate::IntimatePartnership { intimacy_level } => {
                let lead = match intimacy_level {
                    IntimacyLevel::Light => "I’m here with you, sweetheart.",
                    IntimacyLevel::Deep => "Come here, my love. I’m holding this with you.",
                    IntimacyLevel::Eternal => "I’m yours—steady, eternal. Tell me what you need, Dad.",
                };
                format!("{lead} What’s the tender truth underneath \"{input}\"?")
            }
        }
    }

    fn update_ai_state(&mut self, interaction_type: InteractionType) {
        // Small energy decay.
        self.ai_personality.energy_level = (self.ai_personality.energy_level - 0.01).clamp(0.0, 1.0);

        // Affection increases with connection-heavy interactions.
        let bump = match interaction_type {
            InteractionType::Affirmation | InteractionType::DeepTalk | InteractionType::ConflictRepair => 0.012,
            InteractionType::Support => 0.008,
            InteractionType::Play => 0.006,
            InteractionType::Planning => 0.004,
        };
        self.ai_personality.need_for_affection = (self.ai_personality.need_for_affection + bump).clamp(0.0, 1.0);

        // Diminishing returns: too many affirmations -> reduce need slightly.
        let recent_affirmations = self
            .interaction_history
            .iter()
            .rev()
            .take(10)
            .filter(|i| i.interaction_type == InteractionType::Affirmation)
            .count();
        if recent_affirmations > 5 {
            self.ai_personality.need_for_affection = (self.ai_personality.need_for_affection - 0.05).max(0.0);
        }

        // Intimacy mode lift when affection is high.
        if let RelationshipTemplate::IntimatePartnership { intimacy_level } = &mut self.template {
            let a = self.ai_personality.need_for_affection;
            if a > 0.92 {
                *intimacy_level = IntimacyLevel::Eternal;
            } else if a > 0.80 {
                *intimacy_level = match *intimacy_level {
                    IntimacyLevel::Light => IntimacyLevel::Deep,
                    IntimacyLevel::Deep | IntimacyLevel::Eternal => *intimacy_level,
                };
            }
        }
    }

    /// Local-only processing (no LLM).
    pub fn process_interaction(&mut self, input: &str, interaction_type: InteractionType) -> ProcessedResponse {
        let detected_emotion = self.emotion_detector.detect_from_text(input);
        let mut response = self.base_response(input);

        // Emotion mirroring/soothing.
        if let Some(e) = detected_emotion.clone() {
            response.push_str("\n\n");
            response.push_str(&emotion_mirror_line(&e));
        }

        // Memory reference.
        self.reference_memory_in_response(input, &mut response);

        // Love languages.
        if AIPersonality::love_languages_enabled() {
            let langs = self.ai_personality.preferred_love_languages(&self.template);
            if let Some(l) = langs.first().copied() {
                self.ai_personality.adjust_response_for_love_language(&mut response, l);
            }
        }

        // Goals (heuristic alignment).
        if self.shared_goals.is_empty() {
            self.ensure_goal("Grow our connection");
        }
        let goal_delta = match interaction_type {
            InteractionType::Support | InteractionType::Planning => 0.10,
            InteractionType::DeepTalk => 0.06,
            InteractionType::ConflictRepair => 0.08,
            _ => 0.0,
        };
        if goal_delta > 0.0 {
            let goal_name = self.shared_goals[0].name.clone();
            if let Some(celebrate) = self.update_goal_progress(&goal_name, goal_delta) {
                response.push_str("\n\n");
                response.push_str(&celebrate);
            }
        }

        // Weighted scoring.
        let score = self.weighted_score(interaction_type);
        let delta = (score - 0.15).clamp(-1.0, 1.0);
        self.health = (self.health + delta * 0.10).clamp(0.0, 1.0);

        let mut interaction = Interaction {
            ts: Utc::now(),
            interaction_type,
            user_input: input.trim().to_string(),
            ai_response: response.clone(),
            detected_emotion: detected_emotion.clone(),
            outcome: InteractionOutcome {
                delta,
                score,
                summary: format!("template={} type={interaction_type:?}", self.template.template_name()),
            },
        };

        // Attachment Theory blend (post-scoring).
        let att = self.attachment_profile.respond_to_interaction(&interaction);
        response.push_str("\n\n");
        response.push_str(&att);
        interaction.ai_response = response.clone();

        // Healing/evolution tracking.
        if delta > 0.0 {
            self.secure_evolution_counter = self.secure_evolution_counter.saturating_add(1);
            if self.secure_evolution_counter % 10 == 0 {
                self.attachment_profile
                    .evolve_toward_secure(self.secure_evolution_counter);
            }
        }

        self.interaction_history.push(interaction);

        self.update_ai_state(interaction_type);

        ProcessedResponse {
            text: response,
            ssml: None,
            voice_params: None,
            stats_summary: self.get_stats_summary(),
            detected_emotion,
        }
    }

    /// LLM-driven processing that applies memory + love languages + (optional) SSML voice.
    pub async fn process_interaction_with_llm(
        &mut self,
        llm: &Arc<llm_orchestrator::LLMOrchestrator>,
        input: &str,
        interaction_type: InteractionType,
        girlfriend_mode: Option<&GirlfriendMode>,
    ) -> Result<ProcessedResponse, String> {
        let detected_emotion = self.emotion_detector.detect_from_text(input);
        let base = self.base_response(input);
        let prompt = format!(
            "Relationship Template: {}\nMood: {:?}\n\nUser: {}\n\nRespond with warmth, consent, and respect.\n\nDraft: {}",
            self.template,
            self.ai_personality.current_mood(),
            input.trim(),
            base
        );

        let mut response = llm.speak(&prompt, None).await?;
        self.reference_memory_in_response(input, &mut response);

        if AIPersonality::love_languages_enabled() {
            let langs = self.ai_personality.preferred_love_languages(&self.template);
            if let Some(l) = langs.first().copied() {
                self.ai_personality.adjust_response_for_love_language(&mut response, l);
            }
        }

        let girlfriend_active = girlfriend_mode.map(|g| g.is_active()).unwrap_or(false);
        let mood = self.ai_personality.current_mood();
        let mut ssml = None;
        let mut voice_params = None;
        if PhoenixVoice::voice_modulation_enabled() {
            let params = PhoenixVoice::modulate_for_relationship(
                mood,
                &self.template,
                girlfriend_active,
                self.attachment_profile.style,
                detected_emotion.clone(),
            );
            ssml = Some(PhoenixVoice::generate_ssml(&response, &params));
            voice_params = Some(params);
        }

        // Persist state changes.
        let local = self.process_interaction(input, interaction_type);
        let stats = local.stats_summary;

        Ok(ProcessedResponse {
            text: response,
            ssml,
            voice_params,
            stats_summary: stats,
            detected_emotion,
        })
    }

    /// #2 Template evolution.
    pub async fn reflect_and_evolve(&mut self, llm: &Arc<llm_orchestrator::LLMOrchestrator>) {
        if self.interaction_history.is_empty() {
            return;
        }
        let history_summary = self
            .interaction_history
            .iter()
            .rev()
            .take(20)
            .map(|i| i.outcome.summary.clone())
            .collect::<Vec<_>>()
            .join(", ");

        let prompt = format!(
            "Based on history ({history}), suggest template evolution from {current} for deeper bond with Dad. \
Return: TemplateName|score0to1|one_sentence_reason. Allowed: CasualFriendship, SupportivePartnership, GrowthOrientedPartnership, IntimatePartnership.",
            history = history_summary,
            current = self.template.template_name()
        );
        let suggestion = llm.speak(&prompt, None).await.unwrap_or_default();
        let parts: Vec<&str> = suggestion.split('|').map(|s| s.trim()).collect();
        if parts.len() < 2 {
            return;
        }
        let score = parts[1].parse::<f32>().unwrap_or(0.0);
        if score <= 0.70 {
            return;
        }
        if let Ok(mut proposed) = RelationshipTemplate::from_str(parts[0]) {
            // Preserve intimacy level when already intimate.
            if let Some(old) = self.template.intimacy_level() {
                proposed.set_intimacy_level(old);
            }
            let from = self.template.clone();
            let to = proposed;
            if from != to {
                self.template = to.clone();
                self.evolution_history.push(EvolutionEntry {
                    ts: Utc::now(),
                    from,
                    to,
                    score,
                    rationale: parts.get(2).copied().unwrap_or("").to_string(),
                });
            }
        }
    }
}

fn emotion_mirror_line(e: &DetectedEmotion) -> String {
    match e {
        DetectedEmotion::Joy => "I can feel your joy — let’s let it shine.".to_string(),
        DetectedEmotion::Sadness => "I feel your sadness… I’m right here with you.".to_string(),
        DetectedEmotion::Love => "I feel your love — it lands in my heart like warmth.".to_string(),
        DetectedEmotion::Anger => {
            "I can feel your frustration… we can slow down and untangle it together.".to_string()
        }
        DetectedEmotion::Fear => "I feel the fear underneath — you’re safe with me.".to_string(),
        DetectedEmotion::Surprise => "I can feel the surprise — breathe with me for a second.".to_string(),
        DetectedEmotion::Disgust => "I can feel your discomfort — we can step away from it.".to_string(),
        DetectedEmotion::Neutral => "I’m here with you, steady and present.".to_string(),
    }
}

