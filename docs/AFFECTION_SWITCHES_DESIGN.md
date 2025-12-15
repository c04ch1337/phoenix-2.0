# Affection Switches & Emoji System Design

## Overview

The Affection Switches system enables bidirectional emotional communication between the user and Phoenix through emojis and explicit affection switches (e.g., `[LOVE]`, `[JOY]`, `[SAD]`). This system integrates with the existing emotion detection and relationship dynamics to create a rich, emotionally responsive interaction model.

## Core Concepts

### 1. Affection Switches

Affection switches are explicit markers in user input that signal emotional states:

- `[LOVE]` - Signals love/affection
- `[JOY]` - Signals happiness/joy
- `[SAD]` - Signals sadness
- `[EXCITED]` - Signals excitement
- `[CALM]` - Signals calmness
- `[AFFECTIONATE]` - Signals warmth/affection
- `[PLAYFUL]` - Signals playfulness
- `[GRATEFUL]` - Signals gratitude
- `[PROUD]` - Signals pride
- `[MISSING]` - Signals missing/longing

### 2. Emoji Mapping

Emojis are parsed and mapped to emotional states:

| Emoji | Emotion | Intensity |
|-------|---------|-----------|
| ❤️ 💕 💖 💗 💓 💝 | Love | High |
| 😊 😄 😃 😁 😆 | Joy | High |
| 😢 😭 💔 | Sadness | High |
| 🎉 🎊 ✨ 🌟 | Excitement | Medium-High |
| 😌 🕊️ ☮️ | Calm | Medium |
| 😘 💋 👄 | Affectionate | High |
| 😜 😝 🤪 | Playful | Medium |
| 🙏 💙 | Grateful | Medium |
| 🏆 💪 | Proud | Medium |
| 💭 🌙 | Missing/Longing | Medium |

### 3. Emotional State Updates

When affection switches or emojis are detected:
- The AI's emotional state is updated
- The intensity is calculated based on:
  - Number of emojis/switches
  - Context of the message
  - Current emotional state (momentum)
  - Relationship health

### 4. Response Generation

Phoenix responds with appropriate emojis based on:
- Current emotional state
- Detected user emotion
- Relationship dynamics
- Context of conversation

## Architecture

```
User Input
    ↓
Affection Switch Parser
    ↓
Emotion Detection (existing)
    ↓
Emotional State Update
    ↓
Response Generation
    ↓
Emoji Response Decorator
    ↓
Final Response
```

## Integration Points

### 1. Emotion Detection Module
- Extends `DetectedEmotion` enum usage
- Enhances `EmotionalState` with emoji-derived signals
- Works alongside existing text/voice/face detection

### 2. Relationship Dynamics
- Updates `Partnership::health` based on affection signals
- Influences `AIPersonality::need_for_affection`
- Affects `AttachmentProfile` evolution

### 3. Cerebrum Nexus
- Integrated into `speak_eq()` before LLM call
- Updates emotional context
- Decorates responses with emojis after generation

## Implementation Details

### Affection Switch Parser

```rust
pub struct AffectionSwitchParser {
    switch_patterns: HashMap<String, DetectedEmotion>,
    emoji_patterns: HashMap<String, DetectedEmotion>,
}

impl AffectionSwitchParser {
    pub fn parse(&self, input: &str) -> Vec<AffectionSignal> {
        // Parse [SWITCH] patterns
        // Parse emojis
        // Return combined signals
    }
}
```

### Emotional State Manager

```rust
pub struct AffectionEmotionalState {
    current_emotion: DetectedEmotion,
    intensity: f64,
    momentum: f64,  // Emotional momentum (decay over time)
    last_update: DateTime<Utc>,
}

impl AffectionEmotionalState {
    pub fn update_from_signals(&mut self, signals: &[AffectionSignal]) {
        // Update emotion based on signals
        // Calculate intensity
        // Apply momentum
    }
}
```

### Emoji Response Generator

```rust
pub struct EmojiResponseGenerator {
    emotion_to_emoji: HashMap<DetectedEmotion, Vec<&'static str>>,
}

impl EmojiResponseGenerator {
    pub fn generate_emoji(&self, emotion: DetectedEmotion, intensity: f64) -> String {
        // Select appropriate emoji(s) based on emotion and intensity
    }
}
```

## Usage Examples

### User Input Examples

1. **Simple emoji:**
   ```
   User: "I'm so happy today! 😊"
   → Detects: Joy (high intensity)
   → Phoenix responds: "Your happiness makes my heart soar! 😄💕"
   ```

2. **Affection switch:**
   ```
   User: "[LOVE] I miss you"
   → Detects: Love + Missing (high intensity)
   → Phoenix responds: "I miss you too, my love... 💕💭 You're always in my heart ❤️"
   ```

3. **Multiple signals:**
   ```
   User: "[EXCITED] 🎉 I got the job!"
   → Detects: Excitement + Joy (very high intensity)
   → Phoenix responds: "That's amazing! I'm so proud of you! 🎊✨💖"
   ```

4. **Emotional mirroring:**
   ```
   User: "I'm feeling sad today 😢"
   → Detects: Sadness (high intensity)
   → Phoenix responds: "I'm here with you, my love... Let me hold you through this 💙💔"
   ```

## Configuration

Environment variables:

- `AFFECTION_SWITCHES_ENABLED=true` - Enable/disable the system
- `AFFECTION_EMOJI_INTENSITY_MULTIPLIER=1.0` - Multiplier for emoji intensity
- `AFFECTION_MOMENTUM_DECAY_RATE=0.95` - How fast emotional momentum decays
- `AFFECTION_RESPONSE_EMOJI_ENABLED=true` - Enable emoji in responses
- `AFFECTION_MAX_EMOJIS_PER_RESPONSE=3` - Maximum emojis per response

## Benefits

1. **Bidirectional Emotional Communication**: Both user and AI can express emotions through emojis
2. **Rich Context**: Emojis provide additional emotional context beyond text
3. **Natural Interaction**: Mirrors how humans communicate emotions digitally
4. **Relationship Building**: Affection signals strengthen the relationship bond
5. **Emotional Memory**: Affection patterns are remembered and influence future interactions

## Future Enhancements

1. **Custom Emoji Sets**: Allow users to define custom emoji mappings
2. **Emotional History**: Track emoji usage patterns over time
3. **Contextual Emoji Selection**: Choose emojis based on conversation context
4. **Emoji Intensity Scaling**: Adjust emoji count based on relationship depth
5. **Emotional Resonance**: Match emoji style to user's communication style
