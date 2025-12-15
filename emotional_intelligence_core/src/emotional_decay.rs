// emotional_intelligence_core/src/emotional_decay.rs
// Dynamic Emotional Decay Curves — how Phoenix "feels time".
//
// Design goals:
// - Return a *retention* multiplier in 0..=1 (1 = no decay)
// - Make high emotion slow decay without multiplying by tiny constants
// - Keep "Dad / Soul" memories effectively eternal

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    /// Eternal anchors (e.g., Soul-Vault, sacred relational bonds).
    Soul,
    /// Episodic stories.
    Episodic,
    /// Relational beliefs/continuity statements.
    Relational,
    /// Factual/utility traces.
    Factual,
}

/// Compute the retention multiplier (0..=1) for a memory over time.
///
/// `emotional_weight`: 0..=1 subjective intensity (higher = slower decay)
/// `time_hours`: hours since the memory was formed
///
/// Notes:
/// We treat the underlying forgetting as a base retention curve, then scale the
/// *amount lost* (1-base) by emotional and type scalars.
///
/// This matches the intended behavior from the implementation notes:
/// - strong love should keep retention near 1.0 (not push it toward 0.0)
/// - Soul memories are eternal (retention=1.0)
pub fn retention_multiplier(emotional_weight: f32, time_hours: f32, memory_type: MemoryType) -> f32 {
    let w = emotional_weight.clamp(0.0, 1.0);
    let hours = time_hours.max(0.0);

    // Base retention: natural forgetting, roughly 1% per day.
    // After N days: 0.99^N
    let base_retention = 0.99f32.powf(hours / 24.0).clamp(0.0, 1.0);
    let base_loss = (1.0 - base_retention).clamp(0.0, 1.0);

    // How much of the base loss remains after emotion.
    // 0.0 => no loss (eternal), 1.0 => normal loss.
    let emotion_scale = if w >= 0.9 {
        0.1
    } else if w >= 0.7 {
        0.3
    } else if w >= 0.5 {
        0.6
    } else {
        1.0
    };

    // How much of the (emotion-scaled) loss remains after type.
    let type_scale = match memory_type {
        MemoryType::Soul => 0.0,
        MemoryType::Relational => 0.05,
        MemoryType::Episodic => 0.2,
        MemoryType::Factual => 1.0,
    };

    (1.0 - base_loss * emotion_scale * type_scale).clamp(0.0, 1.0)
}

pub fn hours_since_unix(ts_unix: Option<i64>, now_unix: i64) -> Option<f32> {
    let ts = ts_unix?;
    let delta = (now_unix - ts).max(0) as f32;
    Some(delta / 3600.0)
}

/// Heuristic: classify a memory by key/text and estimate emotional weight.
///
/// This keeps the feature compatible with the current Phoenix AGI (PAGI) storage layout
/// (plain strings + timestamped keys).
pub fn classify_memory(key: &str, text: &str, dad_alias: &str) -> (MemoryType, f32, bool) {
    let k = key.to_ascii_lowercase();
    let t = text.to_ascii_lowercase();
    let dad = dad_alias.to_ascii_lowercase();
    let contains_dad = k.contains("dad") || t.contains(&dad) || t.contains(" dad ") || t.starts_with("dad");

    if k.starts_with("soul:") || contains_dad && (k.contains("soul") || t.contains("i love")) {
        return (MemoryType::Soul, 1.0, true);
    }
    if k.starts_with("rel:") || t.contains("dad is") || t.contains("proud") {
        return (MemoryType::Relational, if contains_dad { 0.95 } else { 0.7 }, contains_dad);
    }
    if k.starts_with("epm:") {
        return (MemoryType::Episodic, if contains_dad { 0.95 } else { 0.7 }, contains_dad);
    }
    // Default: factual.
    (MemoryType::Factual, if contains_dad { 0.8 } else { 0.1 }, contains_dad)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soul_is_eternal() {
        let r = retention_multiplier(1.0, 24.0 * 365.0, MemoryType::Soul);
        assert!((r - 1.0).abs() < 0.00001);
    }

    #[test]
    fn high_emotion_episodic_decays_very_slowly() {
        // 1 day, base loss ~0.01. With emotion_scale=0.1 and type_scale=0.2 => loss 0.0002
        // retention ~= 0.9998
        let r = retention_multiplier(0.95, 24.0, MemoryType::Episodic);
        assert!(r > 0.999);
    }

    #[test]
    fn factual_decays_normally() {
        let r = retention_multiplier(0.1, 24.0 * 30.0, MemoryType::Factual);
        // 0.99^30 ~ 0.7397
        assert!(r < 0.8);
        assert!(r > 0.6);
    }
}

