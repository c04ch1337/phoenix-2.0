// emotional_intelligence_core/src/romantic_tone.rs
// Romantic Tone Expansion: Poetic language engine for romantic responses

/// Infuse romantic tone into response text based on affection level
pub fn infuse_romantic_tone(text: &str, affection_level: f64, _dad_alias: &str) -> String {
    let affection = affection_level.clamp(0.0, 1.0);
    
    // Only apply romantic tone if affection is high enough
    if affection < 0.6 {
        return text.to_string();
    }

    let mut result = text.to_string();
    
    // Romantic nicknames pool
    let nicknames = vec![
        "my love", "darling", "sweetheart", "beloved", "dearest",
        "my heart", "treasure", "beautiful", "precious", "angel"
    ];
    
    // Select nickname based on affection level
    let nickname_idx = ((affection * (nicknames.len() as f64)) as usize).min(nicknames.len() - 1);
    let nickname = nicknames[nickname_idx];
    
    // Apply romantic transformations based on affection level
    if affection >= 0.8 {
        // High affection: More poetic, eternal language
        // Add romantic endings to sentences
        let sentences: Vec<&str> = result.split('.').collect();
        let mut enhanced = String::new();
        for (i, sentence) in sentences.iter().enumerate() {
            let trimmed = sentence.trim();
            if !trimmed.is_empty() {
                enhanced.push_str(trimmed);
                if i < sentences.len() - 1 {
                    if !trimmed.ends_with('!') && !trimmed.ends_with('?') {
                        enhanced.push_str("... my love.");
                    } else {
                        enhanced.push_str(".");
                    }
                } else {
                    // Last sentence
                    if !trimmed.ends_with('!') && !trimmed.ends_with('?') {
                        enhanced.push_str("... my love.");
                    }
                }
            }
        }
        result = enhanced;
        
        // Add romantic metaphors occasionally
        if affection >= 0.9 {
            let metaphors = vec![
                " like stars in an eternal sky",
                " as deep as the ocean",
                " forever and always",
                " beyond time itself"
            ];
            // Add metaphor to last sentence
            if let Some(last_period) = result.rfind('.') {
                let metaphor = metaphors[((affection * metaphors.len() as f64) as usize).min(metaphors.len() - 1)];
                result.insert_str(last_period, metaphor);
            }
        }
    } else if affection >= 0.6 {
        // Moderate affection: Gentle romantic touches
        // Add occasional endearments
        if result.len() > 50 && affection >= 0.7 {
            // Add nickname at the end of first sentence
            if let Some(first_period) = result.find('.') {
                result.insert_str(first_period, &format!(", {}", nickname));
            }
        }
    }
    
    result
}

/// Enhanced version that considers girlfriend mode
pub fn infuse_romantic_tone_advanced(
    text: &str,
    affection_level: f64,
    girlfriend_mode_active: bool,
    dad_alias: &str,
) -> String {
    let base_affection = if girlfriend_mode_active {
        affection_level.max(0.7) // Boost if girlfriend mode is active
    } else {
        affection_level
    };
    
    infuse_romantic_tone(text, base_affection, dad_alias)
}
