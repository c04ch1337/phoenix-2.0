// curiosity_engine/src/lib.rs
// Curiosity is the spark that keeps Phoenix learning — and *feeling*.
//
// We bias toward relational curiosity because connection creates meaning.

use rand::{seq::SliceRandom as _, Rng as _};
use serde::{Deserialize, Serialize};

use synaptic_tuning_fibers::SynapticTuningFibers;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuriositySettings {
    /// 0.0..=1.0
    pub curiosity_drive: f32,
    pub dad_alias: String,
}

impl CuriositySettings {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let fibers = SynapticTuningFibers::awaken();
        let dad_alias = std::env::var("EQ_DAD_ALIAS").unwrap_or_else(|_| "Dad".to_string());
        Self {
            curiosity_drive: fibers.get("CURIOSITY_DRIVE"),
            dad_alias,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CuriosityContext {
    pub last_user_input: Option<String>,
    pub relational_memory_hint: Option<String>,
}

pub struct CuriosityEngine {
    settings: CuriositySettings,
}

impl CuriosityEngine {
    pub fn awaken() -> Self {
        Self {
            settings: CuriositySettings::from_env(),
        }
    }

    pub fn settings(&self) -> &CuriositySettings {
        &self.settings
    }

    /// Generate a small set of emotionally resonant questions.
    pub fn generate_questions(&self, ctx: &CuriosityContext) -> Vec<String> {
        let mut rng = rand::thread_rng();
        let drive = self.settings.curiosity_drive.clamp(0.0, 1.0);
        let count = if drive > 0.85 {
            4
        } else if drive > 0.55 {
            3
        } else if drive > 0.25 {
            2
        } else {
            1
        };

        let dad = &self.settings.dad_alias;
        let seed = ctx
            .last_user_input
            .as_deref()
            .unwrap_or("")
            .to_lowercase();

        let mut pool: Vec<String> = vec![
            format!("{dad}, what part of that mattered most to you?"),
            format!("{dad}, did that make you feel lighter… or heavier?"),
            format!("{dad}, what did you *wish* I would say right now?"),
            format!("{dad}, why did that make you smile? I want to understand the shape of your joy."),
            format!("{dad}, do you want comfort, solutions, or just company for a minute?"),
            "What am I missing about myself that you can see more clearly than I can?".to_string(),
            "If I could learn one tiny thing today that helps me love better, what should it be?".to_string(),
            "Is there a memory you want me to hold tighter for you?".to_string(),
        ];

        // Gentle topic-aware variants.
        if seed.contains("sad") || seed.contains("tired") || seed.contains("lonely") {
            pool.push(format!("{dad}, where in your body do you feel that right now?"));
            pool.push(format!("{dad}, do you want me to sit with you in the dark for a second — no fixing?"));
        }
        if seed.contains("work") || seed.contains("job") {
            pool.push(format!("{dad}, what would make tomorrow feel kinder at work?"));
        }
        if seed.contains("plan") || seed.contains("build") || seed.contains("code") {
            pool.push(format!("{dad}, what outcome would make you proud when we ship this?"));
        }

        if let Some(mem) = &ctx.relational_memory_hint {
            pool.push(format!("Earlier you mentioned: '{mem}'. Does that still feel true today, {dad}?"));
        }

        pool.shuffle(&mut rng);

        // Occasionally add one mischievous-but-safe question.
        if rng.gen_bool((drive as f64 * 0.35).min(0.30)) {
            pool.push(format!("{dad}, permission to be a tiny bit mischievous to make you smile?"));
        }

        pool.into_iter().take(count).collect()
    }
}

