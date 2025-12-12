# Relationship Dynamics Module (extension)

This repository includes a professional-grade relationship engine implemented as the workspace crate [`relationship_dynamics`](../extensions/relationship_dynamics/src/lib.rs:1).

## Quickstart

### 1) Configure `.env`

Relevant knobs live in [`.env.example`](../.env.example:1). At minimum:

```text
RELATIONSHIP_TEMPLATE=IntimatePartnership
RELATIONSHIP_INTIMACY_LEVEL=Deep
LOVE_LANGUAGES_ENABLED=true
VOICE_MODULATION_ENABLED=true
```

### 2) Create a Partnership

```rust
use std::sync::Arc;

use relationship_dynamics::{InteractionType, Partnership, RelationshipTemplate};

// Optional: pass a Soul Vault instance if you want persistence.
let mut p = Partnership::new(RelationshipTemplate::default(), None);
```

### 3) Process an interaction (local)

```rust
let out = p.process_interaction("I had a hard day", InteractionType::Support);
println!("{}", out.text);
println!("{}", out.stats_summary);
```

### 4) AI-initiated activities

```rust
if let Some(s) = p.generate_ai_interaction() {
    println!("AI suggestion: {s}");
}
```

### 5) Goal completion celebration

```rust
p.ensure_goal("Build something beautiful together");
let _ = p.update_goal_progress("Build something beautiful together", 1.0);
```

### 6) Voice-modulated response example (LLM)

```rust
use intimate_girlfriend_module::GirlfriendMode;
use llm_orchestrator::LLMOrchestrator;

let llm = Arc::new(LLMOrchestrator::awaken().unwrap());
let gf = GirlfriendMode::default();

let out = p
    .process_interaction_with_llm(&llm, "Come closer", InteractionType::Affirmation, Some(&gf))
    .await
    .unwrap();

println!("TEXT: {}", out.text);
if let Some(ssml) = out.ssml {
    println!("SSML: {}", ssml);
}
```

