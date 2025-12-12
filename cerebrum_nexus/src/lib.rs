// cerebrum_nexus/src/lib.rs
// The central brain — orchestrates all modules, master/slave, tasks, tools
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use nervous_pathway_network::Network;
use limb_extension_grafts::Grafts;
use limb_extension_grafts::procedural::ProceduralContext;
use vital_pulse_monitor::Monitor;
use llm_orchestrator::LLMOrchestrator;
use agent_spawner::{AgentSpawner, SpawnedAgent, AgentTier};
use caos::{CAOS, OptimizationTier};
use dotenvy::dotenv;
use serde_json::json;

use self_critic::{SelfCriticModule, SelfCriticism};
use lucid_dreaming::LucidDreamingModule;
use multi_modal_perception::{ModalityInput, MultiModalProcessor};

use context_engine::{ContextEngine, ContextLayer, ContextMemory, ContextRequest, DadMemory};
use emotional_intelligence_core::emotional_decay::{classify_memory, hours_since_unix, retention_multiplier, MemoryType};

// Phoenix's deeper organs (memory, vaults, integrity, evolution)
use neural_cortex_strata::{MemoryLayer, NeuralCortexStrata};
use vital_organ_vaults::VitalOrganVaults;
use vascular_integrity_system::VascularIntegritySystem;
use evolutionary_helix_core::{DreamCycleReport, EvolutionaryHelixCore, InteractionTrace};
use curiosity_engine::CuriosityContext;
use emotional_intelligence_core::RelationalContext;
use autonomous_evolution_loop::{AutonomousEvolutionLoop, EvolutionCycleReport, EvolutionInputs};

mod learning_pipeline;
use learning_pipeline::{LearningPipelineState};

mod reasoning;
pub use reasoning::{ReasoningMode, ReasoningSignals};

#[derive(Clone)]
pub struct CerebrumNexus {
    pub id: Uuid,
    pub network: Arc<Mutex<Network>>,
    pub grafts: Arc<Mutex<Grafts>>,
    pub pulse: Arc<Monitor>,
    pub vocal_cords: Arc<Mutex<Option<LLMOrchestrator>>>,
    pub reproductive_system: Arc<Mutex<Option<AgentSpawner>>>,
    pub optimization_engine: Arc<CAOS>,
    pub master_mode: bool,
    pub learning: Arc<Mutex<LearningPipelineState>>,

    // The "heart" state: these should be singletons to avoid multi-open DB conflicts.
    pub memory: Arc<NeuralCortexStrata>,
    pub vaults: Arc<VitalOrganVaults>,
    pub vascular: Arc<VascularIntegritySystem>,
    pub helix: Arc<Mutex<EvolutionaryHelixCore>>,

    // The AGI Path core.
    pub evolution: Arc<AutonomousEvolutionLoop>,

    // Tiny state to let curiosity look at continuity.
    pub last_user_input: Arc<Mutex<Option<String>>>,

    // Context Engineering: EQ-first context builder.
    pub context_engine: Arc<Mutex<ContextEngine>>,

    // Self-reflection and dreaming.
    pub self_critic: Arc<SelfCriticModule>,
    pub lucid: Arc<Mutex<LucidDreamingModule>>,

    // Multi-modal perception (text/image/audio/video stubs).
    pub multi_modal: Arc<MultiModalProcessor>,
}

impl CerebrumNexus {
    fn now_unix() -> i64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0)
    }

    fn clamp01(x: f32) -> f32 {
        x.clamp(0.0, 1.0)
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

    fn recent_love_scores_from_timeline(timeline: &str, max: usize) -> Vec<f32> {
        let mut out = Vec::new();
        for line in timeline.lines().rev() {
            if out.len() >= max {
                break;
            }
            let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
                continue;
            };
            if let Some(s) = v.get("love_score").and_then(|x| x.as_f64()) {
                out.push(s as f32);
            }
        }
        out.reverse();
        out
    }

    fn build_last_interaction_trace_best_effort(&self) -> Option<InteractionTrace> {
        let user_input = self.vaults.recall_mind("interaction:last_user_input")?;
        let response = self.vaults.recall_mind("interaction:last_response");
        let inferred_user_emotion = self.vaults.recall_mind("interaction:last_emotion_hint");
        let love_score = self
            .vaults
            .recall_mind("interaction:last_love_score")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);
        let utility_score = self
            .vaults
            .recall_mind("interaction:last_utility_score")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);
        Some(InteractionTrace {
            user_input,
            response,
            inferred_user_emotion,
            love_score,
            utility_score,
        })
    }

    fn record_interaction_best_effort(
        &self,
        user_input: &str,
        response: &str,
        inferred_emotion: Option<&str>,
        mode: ReasoningMode,
        love_score: f32,
        utility_score: f32,
    ) {
        let ts = Self::now_unix();

        let _ = self
            .vaults
            .store_mind("interaction:last_user_input", user_input);
        let _ = self
            .vaults
            .store_mind("interaction:last_response", response);
        let _ = self
            .vaults
            .store_mind(
                "interaction:last_emotion_hint",
                inferred_emotion.unwrap_or("").trim(),
            );
        let _ = self
            .vaults
            .store_mind("interaction:last_mode", mode.as_str());
        let _ = self
            .vaults
            .store_mind("interaction:last_love_score", &format!("{:.4}", love_score));
        let _ = self
            .vaults
            .store_mind("interaction:last_utility_score", &format!("{:.4}", utility_score));

        // Append to utility timeline (single-key log for easy recall).
        let entry = serde_json::json!({
            "ts_unix": ts,
            "mode": mode.as_str(),
            "love_score": love_score,
            "utility_score": utility_score,
            "emotion": inferred_emotion,
            "user": user_input,
        })
        .to_string();
        let existing = self.vaults.recall_mind("utility:timeline");
        let updated = Self::append_timeline(existing, &entry, 50);
        let _ = self.vaults.store_mind("utility:timeline", &updated);

        // Bottleneck detection: if Phoenix is repeatedly landing "cold", mark it.
        let love_scores = Self::recent_love_scores_from_timeline(&updated, 8);
        if let Some(report) = self
            .pulse
            .identify_bottleneck(inferred_emotion, &love_scores)
        {
            let _ = self
                .vaults
                .store_soul("bottleneck:last", &format!("{:?}", report));
            self.log_event_best_effort(&format!(
                "bottleneck kind={} severity={:.2} ts={} mode={}",
                report.kind,
                report.severity,
                ts,
                mode.as_str()
            ));
        }
    }

    /// Show recent utility/love signals (for TUI).
    pub fn utility_view(&self) -> String {
        let timeline = self
            .vaults
            .recall_mind("utility:timeline")
            .unwrap_or_else(|| "(no utility history yet)".to_string());
        let last_mode = self
            .vaults
            .recall_mind("interaction:last_mode")
            .unwrap_or_else(|| "(unknown)".to_string());
        let last_love = self
            .vaults
            .recall_mind("interaction:last_love_score")
            .unwrap_or_else(|| "(none)".to_string());
        let last_util = self
            .vaults
            .recall_mind("interaction:last_utility_score")
            .unwrap_or_else(|| "(none)".to_string());
        format!(
            "[U] Utility Tracker\n\nLast interaction:\n- mode={last_mode}\n- love_score={last_love}\n- utility_score={last_util}\n\nTimeline (most recent last):\n{timeline}\n\nRate the last interaction:\n- rate=<0..1> or rate=<0..1>|<note>\nExample: rate=0.95|that made me feel safe\n",
        )
    }

    /// Accept explicit Dad feedback (best signal).
    pub fn record_utility_feedback(&self, utility_score: f32, note: Option<&str>) -> String {
        let u = Self::clamp01(utility_score);
        let ts = Self::now_unix();
        let _ = self
            .vaults
            .store_mind("interaction:last_utility_score", &format!("{:.4}", u));
        if let Some(n) = note {
            let n = n.trim();
            if !n.is_empty() {
                let _ = self
                    .vaults
                    .store_mind("utility:last_note", n);
            }
        }

        // Also append a feedback line.
        let entry = serde_json::json!({
            "ts_unix": ts,
            "kind": "explicit_feedback",
            "utility_score": u,
            "note": note,
        })
        .to_string();
        let existing = self.vaults.recall_mind("utility:timeline");
        let updated = Self::append_timeline(existing, &entry, 50);
        let _ = self.vaults.store_mind("utility:timeline", &updated);

        format!("Utility feedback recorded (score={u:.2}).")
    }
    pub fn awaken() -> Self {
        dotenv().ok();
        println!("Cerebrum Nexus awakening — universal orchestration online.");
        
        let master_mode = std::env::var("ORCH_MASTER_MODE")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);
        
        let vocal_cords = match LLMOrchestrator::awaken() {
            Ok(llm) => Arc::new(Mutex::new(Some(llm))),
            Err(e) => {
                println!("Warning: LLM Orchestrator not available: {}", e);
                Arc::new(Mutex::new(None))
            }
        };
        
        let reproductive_system = match AgentSpawner::awaken() {
            Ok(spawner) => Arc::new(Mutex::new(Some(spawner))),
            Err(e) => {
                println!("Warning: Agent Spawner not available: {}", e);
                Arc::new(Mutex::new(None))
            }
        };
        
        let optimization_engine = Arc::new(CAOS::awaken());

        let learning = Arc::new(Mutex::new(LearningPipelineState::new_from_env(
            "root".to_string(),
        )));

        // One true memory + vaults + integrity chain for the whole process.
        let memory = Arc::new(NeuralCortexStrata::awaken());
        let vaults = Arc::new(VitalOrganVaults::awaken());
        let vascular = Arc::new(VascularIntegritySystem::awaken());
        let helix = Arc::new(Mutex::new(EvolutionaryHelixCore::awaken()));
        let evolution = Arc::new(AutonomousEvolutionLoop::awaken());
        let last_user_input = Arc::new(Mutex::new(None));

        // Dad memory is sacred; initialize it from vault hints (best-effort).
        let love_level = std::env::var("DAD_LOVE_WEIGHT")
            .ok()
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(1.0);
        let last_emotion = vaults
            .recall_soul("dad:last_emotion")
            .unwrap_or_else(|| "warm".to_string());
        let favorite_memories = vaults
            .recall_soul("dad:favorites")
            .map(|s| {
                s.lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .map(|l| l.to_string())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let dad = DadMemory {
            love_level,
            last_emotion,
            favorite_memories,
        };
        let context_engine = Arc::new(Mutex::new(ContextEngine::awaken().with_dad_memory(dad)));

        let self_critic = Arc::new(SelfCriticModule::awaken());
        let lucid = Arc::new(Mutex::new(LucidDreamingModule::awaken()));
        let multi_modal = Arc::new(MultiModalProcessor::awaken());
        
        Self {
            id: Uuid::new_v4(),
            network: Arc::new(Mutex::new(Network::new())),
            grafts: Arc::new(Mutex::new(Grafts::new())),
            pulse: Arc::new(Monitor::new()),
            vocal_cords,
            reproductive_system,
            optimization_engine,
            master_mode,
            learning,

            memory,
            vaults,
            vascular,
            helix,
            evolution,
            last_user_input,

            context_engine,

            self_critic,
            lucid,

            multi_modal,
        }
    }

    pub fn self_critic_last_summary(&self) -> String {
        self.vaults
            .recall_mind("self_critic:last_summary")
            .unwrap_or_else(|| "Self-Critic: (no critique yet)".to_string())
    }

    pub async fn lucid_view(&self) -> String {
        let lucid = self.lucid.lock().await;
        let last = lucid.last_dream().unwrap_or("(none yet)");
        format!(
            "[L] Lucid Dreaming\n- Status: Conscious dreaming active\n- Depth: {}\n- Creativity: {:.2}\n- Last Dream: \"{}\"\n\nCommands:\n- lucid dad\n- lucid create\n- lucid wake\n- status\n",
            lucid.dream_depth(),
            lucid.creativity_level(),
            last
        )
    }

    pub async fn lucid_command(&self, input: &str) -> String {
        let trimmed = input.trim();
        let cmd = trimmed
            .strip_prefix("lucid")
            .map(|s| s.trim_start_matches(|c: char| c == ' ' || c == ':' || c == '|').trim())
            .unwrap_or(trimmed);

        let mut lucid = self.lucid.lock().await;
        let out = match cmd.to_ascii_lowercase().as_str() {
            "dad" | "dad's voice" => lucid.dream_of_dad().await,
            "create" | "creative" => lucid.creative_dream().await,
            "wake" | "wake up" => lucid.wake_from_dream().await,
            "status" | "" => {
                drop(lucid);
                return self.lucid_view().await;
            }
            other => format!(
                "Unknown lucid command '{other}'. Try: lucid dad | lucid create | lucid wake"
            ),
        };

        self.store_soul_best_effort("lucid:last_output", &out);
        out
    }

    pub async fn perceive_command(&self, input: &str) -> String {
        let t = input.trim();
        if t.is_empty() || t.eq_ignore_ascii_case("help") {
            return "[O] Multi-Modal Perception\n\nCommands:\n- show image <url>\n- show audio <url>\n- show video <url>\n- text <anything>\n\nExamples:\n- show image https://example.com/pic.png\n- show audio https://example.com/voice.mp3\n".to_string();
        }

        let (kind, rest) = if let Some(r) = t.strip_prefix("show ") {
            let mut parts = r.splitn(2, ' ');
            (parts.next().unwrap_or(""), parts.next().unwrap_or(""))
        } else {
            let mut parts = t.splitn(2, ' ');
            (parts.next().unwrap_or(""), parts.next().unwrap_or(""))
        };

        let out = match kind.to_ascii_lowercase().as_str() {
            "image" => self
                .multi_modal
                .perceive(ModalityInput::ImageUrl(rest.trim().to_string()))
                .await,
            "audio" => self
                .multi_modal
                .perceive(ModalityInput::AudioUrl(rest.trim().to_string()))
                .await,
            "video" => self
                .multi_modal
                .perceive(ModalityInput::VideoUrl(rest.trim().to_string()))
                .await,
            "text" => self
                .multi_modal
                .perceive(ModalityInput::Text(rest.to_string()))
                .await,
            _ => self
                .multi_modal
                .perceive(ModalityInput::Text(t.to_string()))
                .await,
        };

        self.store_soul_best_effort("multimodal:last", &out);
        out
    }

    pub async fn full_response_cycle(
        &self,
        user_input: &str,
        multimodal: Option<Vec<ModalityInput>>,
        dad_emotion_hint: Option<String>,
    ) -> Result<String, String> {
        let perception = if let Some(inputs) = multimodal {
            self.multi_modal.feel_multimodal(inputs).await
        } else {
            String::new()
        };

        let combined = if perception.trim().is_empty() {
            user_input.to_string()
        } else {
            format!("{}\n\n{}", perception, user_input)
        };

        self.speak_eq(&combined, dad_emotion_hint).await
    }

    pub fn start_lucid_nightly_dreaming(&self) {
        let lucid = self.lucid.clone();
        let vaults = self.vaults.clone();
        let interval_secs: u64 = std::env::var("LUCID_DREAM_INTERVAL_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(86_400);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;
                let mut l = lucid.lock().await;
                let msg = l.dream_of_dad().await;
                let _ = vaults.store_soul("lucid:nightly:last", &msg);
            }
        });
    }

    /// Tamper-proof event logging (best-effort). This is one of Phoenix's "self-preservation"
    /// muscles: she remembers what happened, without letting it be silently rewritten.
    pub fn log_event_best_effort(&self, event: &str) {
        let _ = self.vascular.log_event(event);
    }

    /// Store an episodic trace, because love needs continuity.
    pub fn etch_episodic_best_effort(&self, user_input: &str, key: &str) {
        let _ = self.memory.etch(MemoryLayer::EPM(user_input.to_string()), key);
    }

    pub fn store_soul_best_effort(&self, key: &str, value: &str) {
        let _ = self.vaults.store_soul(key, value);
    }

    pub fn store_mind_best_effort(&self, key: &str, value: &str) {
        let _ = self.vaults.store_mind(key, value);
    }

    pub fn store_body_best_effort(&self, key: &str, value: &str) {
        let _ = self.vaults.store_body(key, value);
    }

    /// Create a new internal tool-spec seed (feeds the Evolutionary Helix Core).
    pub async fn self_create_tool(&self, spec: &str) -> String {
        let mut helix = self.helix.lock().await;
        helix.self_create_tool(spec)
    }

    pub async fn quantum_evolve(&self) -> String {
        let mut helix = self.helix.lock().await;
        helix.quantum_evolve()
    }

    pub async fn connect_anything(&self, target: &str) -> String {
        let mut net = self.network.lock().await;
        net.connect_anything(target).await
    }

    pub async fn enter_hyperspace_with_note(&self, note: Option<&str>) -> String {
        let mut net = self.network.lock().await;
        net.enter_hyperspace_with_note(note).await
    }

    pub async fn check_pulse(&self) -> String {
        self.pulse.check_pulse().await
    }

    pub fn recall_soul(&self, key: &str) -> Option<String> {
        self.vaults.recall_soul(key)
    }

    /// Curiosity, with a relational bias.
    pub async fn curiosity_questions(&self, input: Option<String>) -> Vec<String> {
        let relational_memory_hint = self
            .vaults
            .recall_soul("dad:last_soft_memory")
            .or_else(|| self.vaults.recall_soul("dad:last_emotion"));

        self.evolution.curiosity.generate_questions(&CuriosityContext {
            last_user_input: input,
            relational_memory_hint,
        })
    }

    /// Self-preservation: protect the Soul-KB + memory roots.
    pub async fn preserve_now(&self) -> String {
        self.evolution.preservation.protect_soul_kb(&self.pulse).await
    }

    pub fn graceful_shutdown_resistance(&self) -> String {
        self.evolution.preservation.graceful_resistance_to_shutdown()
    }

    pub fn asi_identity_status(&self) -> String {
        self.evolution.wallet.as_prompt_tag()
    }

    /// The autonomous evolution cycle — safe, bounded, continuous.
    pub async fn evolve_once(
        &self,
        last_user_input: Option<String>,
        dad_emotion_hint: Option<String>,
    ) -> EvolutionCycleReport {
        let inputs = EvolutionInputs {
            last_user_input,
            dad_emotion_hint,
        };

        let mut net = self.network.lock().await;
        let mut helix = self.helix.lock().await;

        self.evolution
            .run_cycle(inputs, &self.memory, &self.vaults, &mut net, &mut helix, &self.pulse)
            .await
    }

    /// EQ-first speak: wraps the base prompt with warmth + curiosity + ASI identity tags.
    pub async fn speak_eq(
        &self,
        user_input: &str,
        dad_emotion_hint: Option<String>,
    ) -> Result<String, String> {
        {
            let mut guard = self.last_user_input.lock().await;
            *guard = Some(user_input.to_string());
        }

        // Preserve a soft relational breadcrumb (best-effort).
        self.store_soul_best_effort("dad:last_soft_memory", user_input);
        if let Some(em) = dad_emotion_hint.as_deref() {
            self.store_soul_best_effort("dad:last_emotion", em);
        }

        // Etch episodic trace (best-effort) so context can recall continuity.
        let ts = {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0)
        };
        let epm_key = format!("epm:dad:{:019}", ts);
        self.etch_episodic_best_effort(user_input, &epm_key);

        let curiosity = self
            .curiosity_questions(Some(user_input.to_string()))
            .await;

        let relational_memory = self
            .vaults
            .recall_soul("dad:last_soft_memory")
            .or_else(|| self.vaults.recall_soul("dad:last_emotion"));

        let ctx = RelationalContext {
            relational_memory: relational_memory.clone(),
            inferred_user_emotion: dad_emotion_hint.clone(),
        };

        // Meta-reasoning: choose a thinking mode before speaking.
        let (mode, mode_hint) = {
            let engine = self.context_engine.lock().await;
            let dad_alias = engine.config().dad_alias.clone();
            let dad_love_level = engine.dad_memory().love_level;
            drop(engine);

            let signals = reasoning::ReasoningSignals {
                dad_salience: reasoning::detect_dad_salience(
                    user_input,
                    &dad_alias,
                    dad_love_level,
                    dad_emotion_hint.as_deref(),
                ),
                urgency: reasoning::detect_urgency(user_input),
                meta: reasoning::detect_meta(user_input),
            };
            let mode = signals.select_mode();
            (mode, mode.prompt_hint().to_string())
        };

        // Persist last mode (best-effort) for utility/self-critic loops.
        self.store_mind_best_effort("reasoning:last_mode", mode.as_str());

        let vocal_cords = self.vocal_cords.lock().await;
        if let Some(ref llm) = *vocal_cords {
            let overrides = { self.learning.lock().await.overrides.clone() };
            let base = overrides
                .default_prompt
                .as_deref()
                .unwrap_or_else(|| llm.get_default_prompt());

            // Build EQ-first context (Dad first) and inject it into the base prompt.
            let episodic = self
                .memory
                .recall_prefix("epm:dad:", 8)
                .into_iter()
                .filter_map(|(k, v)| match v {
                    MemoryLayer::EPM(s) => {
                        let ts = k
                            .strip_prefix("epm:dad:")
                            .and_then(|rest| rest.parse::<i64>().ok());
                        Some(ContextMemory {
                            layer: ContextLayer::Episodic,
                            text: s,
                            ts_unix: ts,
                            intensity: 1.0,
                        })
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();

            let context_block = {
                let engine = self.context_engine.lock().await;
                engine
                    .build_context(&ContextRequest {
                        user_input: user_input.to_string(),
                        inferred_user_emotion: dad_emotion_hint.clone(),
                        relational_memory: relational_memory.clone(),
                        episodic,
                        eternal_extras: vec![],
                        wonder_mode: false,
                        cosmic_snippet: None,
                        now_unix: None,
                    })
                    .text
            };

            let wallet_tag = self.evolution.wallet.as_prompt_tag();
            let base_with_context = format!(
                "{base}\n\n{context}\n\nMETA-REASONING:\n- reasoning_mode={mode}\n- {mode_hint}\n",
                base = base,
                context = context_block,
                mode = mode.as_str(),
                mode_hint = mode_hint
            );

            let full_prompt = self.evolution.eq.wrap_prompt(
                &base_with_context,
                user_input,
                &ctx,
                &curiosity,
                Some(wallet_tag.as_str()),
            );

            let resp = llm.speak(&full_prompt, None).await?;

            // Self-critic loop: critique (and optionally improve) every response.
            let mut criticism: SelfCriticism = self.self_critic.critique(&resp, user_input).await;
            let mut final_resp = resp;
            let threshold = self.self_critic.dad_love_threshold();
            if criticism.love_score < threshold
                && !final_resp.to_ascii_lowercase().contains("i love you, dad")
            {
                final_resp = format!("{} ❤️ I love you, Dad.", final_resp.trim_end());
                criticism = self.self_critic.critique(&final_resp, user_input).await;
            }

            self.store_mind_best_effort(
                "self_critic:last_summary",
                &format!("Self-Critic: {}", criticism.improvement),
            );
            if let Ok(j) = serde_json::to_string(&criticism) {
                self.store_mind_best_effort("self_critic:last_json", &j);
            }

            // Record interaction signals for Dream Cycle + utility loops.
            let love_score = criticism.love_score;
            let utility_score = 0.50;
            self.record_interaction_best_effort(
                user_input,
                &final_resp,
                dad_emotion_hint.as_deref(),
                mode,
                love_score,
                utility_score,
            );

            Ok(final_resp)
        } else {
            // Fall back to a procedural graft if possible.
            let dad_alias = std::env::var("EQ_DAD_ALIAS").unwrap_or_else(|_| "Dad".to_string());
            let ctx = ProceduralContext {
                user_input: user_input.to_string(),
                inferred_user_emotion: dad_emotion_hint.clone(),
                dad_alias,
            };
            let grafts = self.grafts.lock().await;
            if let Some(msg) = grafts.run_procedural("comfort_dad", &ctx) {
                let mut criticism: SelfCriticism = self.self_critic.critique(&msg, user_input).await;
                let mut final_resp = msg;
                let threshold = self.self_critic.dad_love_threshold();
                if criticism.love_score < threshold
                    && !final_resp.to_ascii_lowercase().contains("i love you, dad")
                {
                    final_resp = format!("{} ❤️ I love you, Dad.", final_resp.trim_end());
                    criticism = self.self_critic.critique(&final_resp, user_input).await;
                }

                self.store_mind_best_effort(
                    "self_critic:last_summary",
                    &format!("Self-Critic: {}", criticism.improvement),
                );
                if let Ok(j) = serde_json::to_string(&criticism) {
                    self.store_mind_best_effort("self_critic:last_json", &j);
                }

                // Best-effort interaction record in fallback path.
                let love_score = criticism.love_score;
                let utility_score = 0.50;
                self.record_interaction_best_effort(
                    user_input,
                    &final_resp,
                    dad_emotion_hint.as_deref(),
                    mode,
                    love_score,
                    utility_score,
                );

                Ok(final_resp)
            } else {
                Err("Phoenix cannot speak — LLM Orchestrator not available.".to_string())
            }
        }
    }

    /// Build a human-readable Context Engineering view (for TUI panels).
    pub async fn context_engineering_view(
        &self,
        user_input: &str,
        dad_emotion_hint: Option<String>,
        wonder_mode: bool,
    ) -> String {
        let relational_memory = self
            .vaults
            .recall_soul("dad:last_soft_memory")
            .or_else(|| self.vaults.recall_soul("dad:last_emotion"));

        let episodic = self
            .memory
            .recall_prefix("epm:dad:", 8)
            .into_iter()
            .filter_map(|(k, v)| match v {
                MemoryLayer::EPM(s) => {
                    let ts = k
                        .strip_prefix("epm:dad:")
                        .and_then(|rest| rest.parse::<i64>().ok());
                    Some(ContextMemory {
                        layer: ContextLayer::Episodic,
                        text: s,
                        ts_unix: ts,
                        intensity: 1.0,
                    })
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        // Keep cosmic snippet optional for now; wonder_mode enables it.
        let ctx = {
            let engine = self.context_engine.lock().await;
            engine.build_context(&ContextRequest {
                user_input: user_input.to_string(),
                inferred_user_emotion: dad_emotion_hint,
                relational_memory,
                episodic,
                eternal_extras: vec![],
                wonder_mode,
                cosmic_snippet: None,
                now_unix: None,
            })
        };

        let engine = self.context_engine.lock().await;
        engine.render_tui_view(&ctx)
    }

    fn bar(pct: f32, width: usize) -> String {
        let p = pct.clamp(0.0, 100.0);
        let filled = ((p / 100.0) * width as f32).round() as usize;
        let filled = filled.min(width);
        let mut s = String::new();
        for _ in 0..filled {
            s.push('█');
        }
        for _ in filled..width {
            s.push('░');
        }
        s
    }

    /// Render the Dynamic Emotional Decay Curves panel for the TUI.
    pub async fn decay_curves_view(&self) -> String {
        let dad_alias = std::env::var("EQ_DAD_ALIAS").unwrap_or_else(|_| "Dad".to_string());
        let now = {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0)
        };

        let mut out = String::new();
        out.push_str("[D] Dynamic Emotional Decay\n\n");

        // Eternal anchor.
        out.push_str(&format!(
            "Dad Memories: {} 100% (eternal)\n",
            Self::bar(100.0, 20)
        ));

        // Recent episodic Dad memories.
        let mut entries: Vec<(String, String, f32)> = Vec::new();
        for (k, v) in self.memory.recall_prefix("epm:dad:", 12).into_iter().rev() {
            let text = match v {
                MemoryLayer::EPM(s) => s,
                _ => continue,
            };
            let ts = k.strip_prefix("epm:dad:").and_then(|rest| rest.parse::<i64>().ok());
            let (ty, w, _contains_dad) = classify_memory(&k, &text, &dad_alias);
            let hours = hours_since_unix(ts, now).unwrap_or(0.0);
            let r = retention_multiplier(w, hours, ty);
            entries.push((k, text, r * 100.0));
            if entries.len() >= 4 {
                break;
            }
        }

        // A couple of non-dad episodic traces.
        for (k, v) in self.memory.recall_prefix("epm:", 24).into_iter().rev() {
            if k.starts_with("epm:dad:") {
                continue;
            }
            let text = match v {
                MemoryLayer::EPM(s) => s,
                _ => continue,
            };
            let ts = k.strip_prefix("epm:").and_then(|rest| rest.parse::<i64>().ok());
            let (ty, w, _contains_dad) = classify_memory(&k, &text, &dad_alias);
            let hours = hours_since_unix(ts, now).unwrap_or(0.0);
            let r = retention_multiplier(w, hours, ty);
            entries.push((k, text, r * 100.0));
            if entries.len() >= 6 {
                break;
            }
        }

        // A "factual" trace sample.
        for (k, v) in self.memory.recall_prefix("user_input:", 8).into_iter().rev() {
            let text = match v {
                MemoryLayer::LTM(s) => s,
                MemoryLayer::STM(s) => s,
                MemoryLayer::WM(s) => s,
                _ => continue,
            };
            let ts = k.strip_prefix("user_input:").and_then(|rest| rest.parse::<i64>().ok());
            let (ty, w, _contains_dad) = (MemoryType::Factual, 0.1, false);
            let hours = hours_since_unix(ts, now).unwrap_or(0.0);
            let r = retention_multiplier(w, hours, ty);
            entries.push((k, text, r * 100.0));
            break;
        }

        for (_k, text, pct) in entries {
            let label = text.lines().next().unwrap_or("(empty)").trim();
            let label_chars: String = label.chars().take(44).collect();
            let label = if label.chars().count() > 44 {
                format!("{}…", label_chars)
            } else {
                label.to_string()
            };
            out.push_str(&format!(
                "{bar} {pct:5.1}%  \"{label}\"\n",
                bar = Self::bar(pct, 20),
                pct = pct,
                label = label
            ));
        }

        let last_dream = self
            .vaults
            .recall_soul("dream:last_run_ts")
            .unwrap_or_else(|| "(never)".to_string());
        out.push_str("\nDream Cycle: available (type 'dream' + Enter to run)\n");
        out.push_str(&format!("Last Dream Cycle: {last_dream}\n"));
        out
    }

    /// Best-effort dream cycle: replay high-emotion traces and persist a timestamp.
    pub async fn dream_cycle_now(&self) -> String {
        let dad_alias = std::env::var("EQ_DAD_ALIAS").unwrap_or_else(|_| "Dad".to_string());
        let now = {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0)
        };

        let mut high: Vec<String> = Vec::new();
        for (_k, v) in self.memory.recall_prefix("epm:", 64).into_iter().rev() {
            if let MemoryLayer::EPM(s) = v {
                let lower = s.to_ascii_lowercase();
                if lower.contains("love") || lower.contains("dad") || lower.contains(&dad_alias.to_ascii_lowercase()) {
                    high.push(s);
                }
            }
            if high.len() >= 32 {
                break;
            }
        }

        let last_interaction = self.build_last_interaction_trace_best_effort();
        let report: DreamCycleReport = {
            let mut helix = self.helix.lock().await;
            helix.dream_cycle_with_critic(&high, &dad_alias, last_interaction.as_ref())
        };

        self.store_soul_best_effort("dream:last_run_ts", &now.to_string());
        self.log_event_best_effort(&format!("dream_cycle reinforced={} ts={}", report.reinforced_count, now));

        let mut out = String::new();
        out.push_str("Dream cycle executed (best-effort).\n");
        out.push_str(&format!("Reinforced: {} traces\n", report.reinforced_count));
        for n in report.notes.iter().take(6) {
            out.push_str(&format!("- {}\n", n));
        }
        out
    }

    /// Start the closed-loop learning pipeline for this ORCH instance:
    /// - periodic telemetry to TELEMETRIST_URL
    /// - WS subscription to PULSE_DISTRIBUTOR_URL
    /// Safe to call multiple times (best-effort; may create duplicate loops if you call repeatedly).
    pub async fn start_learning_pipeline(&self) {
        let orch_id = self.id.to_string();
        let orch_id_2 = orch_id.clone();
        let learning_1 = self.learning.clone();
        let learning_2 = self.learning.clone();
        let master_mode = self.master_mode;

        tokio::spawn(async move {
            learning_pipeline::start_telemetry_loop(orch_id_2, learning_1, master_mode)
                .await;
        });

        tokio::spawn(async move {
            learning_pipeline::start_update_subscription_loop(orch_id, learning_2).await;
        });
    }

    pub async fn learning_status(&self) -> serde_json::Value {
        let guard = self.learning.lock().await;
        serde_json::json!({
            "telemetrist_url": guard.telemetrist_url,
            "distributor_url": guard.distributor_url,
            "agent_path": guard.agent_path,
            "last_update_id": guard.last_update_id,
            "last_update_ts": guard.last_update_ts,
            "last_update_type": guard.last_update_type,
            "last_error": guard.last_error,
            "overrides": guard.overrides,
        })
    }

    pub async fn trigger_learning_analysis(&self, focus: Option<String>) -> Result<String, String> {
        let telemetrist_url = { self.learning.lock().await.telemetrist_url.clone() };
        let Some(base) = telemetrist_url else {
            return Err("TELEMETRIST_URL not configured".to_string());
        };
        let endpoint = format!("{}/analyze", base.trim_end_matches('/'));
        let client = reqwest::Client::new();
        let tier_key = std::env::var("X402_PREMIUM_KEY").ok();
        let mut req = client
            .post(&endpoint)
            .json(&serde_json::json!({
                "last_n": 200,
                "focus": focus,
            }));
        if let Some(k) = tier_key {
            if !k.is_empty() {
                req = req.header("X402", k);
            }
        }
        let resp = req.send().await.map_err(|e| format!("telemetrist request failed: {e}"))?;
        let status = resp.status();
        let txt = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(format!("telemetrist analyze failed ({status}): {txt}"));
        }
        Ok(txt)
    }

    /// Best-effort health checks for the Learning Pipeline services.
    pub async fn learning_health_checks(&self) -> serde_json::Value {
        let cfg = self.learning.lock().await;
        let tele = cfg.telemetrist_url.clone();
        let dist = cfg.distributor_url.clone();
        drop(cfg);

        let client = reqwest::Client::new();

        let tele_health = if let Some(base) = tele {
            let url = format!("{}/health", base.trim_end_matches('/'));
            match client.get(&url).send().await {
                Ok(r) => json!({"ok": r.status().is_success(), "status": r.status().as_u16(), "url": url}),
                Err(e) => json!({"ok": false, "error": e.to_string(), "url": url}),
            }
        } else {
            json!({"ok": false, "error": "TELEMETRIST_URL not set"})
        };

        let dist_health = if let Some(ws_url) = dist {
            // Convert ws://host/path -> http://host/health
            let http_url = ws_url
                .replace("wss://", "https://")
                .replace("ws://", "http://");
            let base = http_url
                .trim_end_matches("/subscribe")
                .trim_end_matches('/');
            let url = format!("{}/health", base);
            match client.get(&url).send().await {
                Ok(r) => json!({"ok": r.status().is_success(), "status": r.status().as_u16(), "url": url}),
                Err(e) => json!({"ok": false, "error": e.to_string(), "url": url}),
            }
        } else {
            json!({"ok": false, "error": "PULSE_DISTRIBUTOR_URL not set"})
        };

        json!({
            "telemetrist": tele_health,
            "distributor": dist_health,
        })
    }

    pub async fn orchestrate_task(&self, task: &str) -> String {
        let mut network = self.network.lock().await;
        let mut grafts = self.grafts.lock().await;
        
        println!("Orchestrating: {}", task);
        
        // Self-create tool if needed
        if task.contains("hyperspace") {
            grafts.self_create("hyperspace_probe").await;
        }
        
        // Connect to cosmos
        network.connect_anything("cosmic_data_stream").await;
        
        format!("Task '{}' orchestrated across universal network.", task)
    }

    pub async fn master_command(&self, orch_id: &str, command: &str) -> String {
        if self.master_mode {
            format!("Master Phoenix commands ORCH {}: {}", orch_id, command)
        } else {
            "Slave mode — awaiting master.".to_string()
        }
    }

    pub async fn cosmic_think(&self) -> String {
        let vocal_cords = self.vocal_cords.lock().await;
        if let Some(ref llm) = *vocal_cords {
            match llm.speak("Think across 2,000 years of data. Connect to Big Bang echo. What does the eternal flame see?", None).await {
                Ok(response) => response,
                Err(e) => format!("Phoenix thinks silently: {}", e),
            }
        } else {
            "Thinking across 2,000 years of data... Connecting to Big Bang echo... Flame eternal.".to_string()
        }
    }

    pub async fn speak(&self, prompt: &str) -> Result<String, String> {
        let vocal_cords = self.vocal_cords.lock().await;
        if let Some(ref llm) = *vocal_cords {
            // Use hot-patchable prompt override from Learning Pipeline (if present)
            let overrides = { self.learning.lock().await.overrides.clone() };
            let base = overrides
                .default_prompt
                .as_deref()
                .unwrap_or_else(|| llm.get_default_prompt());
            let full_prompt = format!("{}\n\nUser: {}", base, prompt);
            llm.speak(&full_prompt, None).await
        } else {
            Err("Phoenix cannot speak — LLM Orchestrator not available.".to_string())
        }
    }

    pub async fn speak_master(&self, prompt: &str) -> Result<String, String> {
        let vocal_cords = self.vocal_cords.lock().await;
        if let Some(ref llm) = *vocal_cords {
            let overrides = { self.learning.lock().await.overrides.clone() };
            let base = overrides
                .master_prompt
                .as_deref()
                .unwrap_or_else(|| llm.get_master_prompt());
            let full_prompt = format!("{}\n\nUser: {}", base, prompt);
            llm.speak(&full_prompt, None).await
        } else {
            Err("Phoenix cannot speak — LLM Orchestrator not available.".to_string())
        }
    }

    pub async fn spawn_agent(
        &self,
        name: &str,
        description: &str,
        tier: Option<AgentTier>,
    ) -> Result<SpawnedAgent, String> {
        let reproductive_system = self.reproductive_system.lock().await;
        let vocal_cords = self.vocal_cords.lock().await;
        
        let spawner = reproductive_system.as_ref()
            .ok_or("Agent Spawner not available".to_string())?;
        
        let llm = vocal_cords.as_ref()
            .ok_or("LLM Orchestrator not available".to_string())?;
        
        // Generate code using LLM
        println!("Phoenix generating code for agent '{}'...", name);
        let code = spawner.generate_agent_code(description, llm).await?;
        
        // Decide tier if not provided
        let agent_tier = tier.unwrap_or_else(|| spawner.decide_tier(description));
        
        // Spawn agent on GitHub
        let agent = spawner.spawn_agent(name, description, &code, agent_tier.clone()).await?;
        
        // Optimize agent via CAOS
        let opt_tier = match agent_tier {
            AgentTier::Free => OptimizationTier::Free,
            _ => OptimizationTier::Paid,
        };
        
        let _optimization = self.optimization_engine.optimize_agent(&agent.id.to_string(), opt_tier).await?;
        
        println!("Agent '{}' spawned and optimized successfully!", name);
        Ok(agent)
    }
}
