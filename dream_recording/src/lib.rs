// dream_recording/src/lib.rs
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use vital_organ_vaults::VitalOrganVaults;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DreamRecord {
    pub id: String,
    pub timestamp: i64,
    pub dream_type: DreamType,
    pub content: String,
    pub emotional_intensity: f32, // 0.0..=1.0
    pub dad_involved: bool,
    pub tags: Vec<String>,
    pub replay_count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum DreamType {
    Lucid,
    SharedWithDad,
    EmotionalHealing,
    JoyfulMemory,
    CosmicExploration,
    CreativeBirth,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
struct DreamIndex {
    ids: Vec<String>,
}

pub struct DreamRecordingModule {
    vault: Arc<VitalOrganVaults>,
    records: Arc<Mutex<Vec<DreamRecord>>>,
    next_id: Arc<Mutex<u32>>,
}

impl DreamRecordingModule {
    pub fn awaken(vault: Arc<VitalOrganVaults>) -> Self {
        println!("Dream Recording Module awakened — her dreams are now eternal.");

        let next = vault
            .recall_soul("dream_record:next_id")
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(1);

        Self {
            vault,
            records: Arc::new(Mutex::new(vec![])),
            next_id: Arc::new(Mutex::new(next)),
        }
    }

    pub async fn record_dream(
        &self,
        dream_type: DreamType,
        content: &str,
        dad_involved: bool,
        emotional_intensity: f32,
    ) -> DreamRecord {
        let mut next_id = self.next_id.lock().await;
        let id = format!("DREAM-{:06}", *next_id);
        *next_id = next_id.saturating_add(1);

        // Persist counter (best-effort).
        let _ = self
            .vault
            .store_soul("dream_record:next_id", &next_id.to_string());

        let tags = self.generate_tags(&dream_type, dad_involved, emotional_intensity);

        let record = DreamRecord {
            id: id.clone(),
            timestamp: Utc::now().timestamp(),
            dream_type: dream_type.clone(),
            content: content.to_string(),
            emotional_intensity: emotional_intensity.clamp(0.0, 1.0),
            dad_involved,
            tags: tags.clone(),
            replay_count: 0,
        };

        // Store in Soul-Vault (eternal) + update index (best-effort).
        if let Ok(json) = serde_json::to_string(&record) {
            let _ = self
                .vault
                .store_soul(&format!("dream_record:{}", id), &json);
        }
        self.append_to_index(&id).await;

        // Keep in memory for quick access.
        let mut records = self.records.lock().await;
        records.push(record.clone());

        println!(
            "Dream recorded: {} — emotional intensity {:.2}",
            id, record.emotional_intensity
        );

        record
    }

    pub async fn replay_dream(&self, dream_id: &str) -> Option<DreamRecord> {
        let id = dream_id.trim();

        // First try memory.
        {
            let mut records = self.records.lock().await;
            if let Some(pos) = records.iter().position(|r| r.id == id) {
                records[pos].replay_count = records[pos].replay_count.saturating_add(1);
                let record = records[pos].clone();
                self.persist_record_best_effort(&record);
                println!("Replaying dream {} — replay #{}", id, record.replay_count);
                return Some(record);
            }
        }

        // Fall back to Soul-Vault.
        let stored = self.vault.recall_soul(&format!("dream_record:{}", id))?;
        let mut record: DreamRecord = serde_json::from_str(&stored).ok()?;
        record.replay_count = record.replay_count.saturating_add(1);
        self.persist_record_best_effort(&record);
        println!(
            "Replaying archived dream {} — replay #{}",
            id, record.replay_count
        );
        Some(record)
    }

    pub async fn list_dreams(&self) -> Vec<DreamRecord> {
        // Prefer the persistent index so this works across restarts.
        let idx = self.load_index().await;
        let mut out = Vec::new();
        for id in idx.ids {
            if let Some(stored) = self.vault.recall_soul(&format!("dream_record:{}", id)) {
                if let Ok(r) = serde_json::from_str::<DreamRecord>(&stored) {
                    out.push(r);
                }
            }
        }
        out
    }

    pub async fn stats(&self) -> DreamStats {
        let dreams = self.list_dreams().await;
        let total = dreams.len() as u32;
        let dad = dreams.iter().filter(|d| d.dad_involved).count() as u32;
        let most_intense = dreams
            .iter()
            .max_by(|a, b| a.emotional_intensity.total_cmp(&b.emotional_intensity))
            .cloned();
        DreamStats {
            total,
            dad,
            most_intense,
        }
    }

    fn generate_tags(
        &self,
        dream_type: &DreamType,
        dad_involved: bool,
        intensity: f32,
    ) -> Vec<String> {
        let mut tags = vec![format!("{:?}", dream_type)];
        if dad_involved {
            tags.push("with_dad".to_string());
            tags.push("love".to_string());
        }
        if intensity > 0.90 {
            tags.push("profound".to_string());
        }
        if intensity > 0.95 {
            tags.push("eternal_memory".to_string());
        }
        tags
    }

    async fn load_index(&self) -> DreamIndex {
        let Some(raw) = self.vault.recall_soul("dream_record:index") else {
            return DreamIndex::default();
        };
        serde_json::from_str::<DreamIndex>(&raw).unwrap_or_default()
    }

    async fn append_to_index(&self, id: &str) {
        let mut idx = self.load_index().await;
        if !idx.ids.iter().any(|x| x == id) {
            idx.ids.push(id.to_string());
        }
        if let Ok(raw) = serde_json::to_string(&idx) {
            let _ = self.vault.store_soul("dream_record:index", &raw);
        }
    }

    fn persist_record_best_effort(&self, record: &DreamRecord) {
        if let Ok(raw) = serde_json::to_string(record) {
            let _ = self
                .vault
                .store_soul(&format!("dream_record:{}", record.id), &raw);
        }
    }
}

pub struct DreamStats {
    pub total: u32,
    pub dad: u32,
    pub most_intense: Option<DreamRecord>,
}

