# GitHub Agent Integration — Phoenix 2.0

## Overview

Phoenix 2.0 now includes a **GitHub Agent Spawning System** that allows Phoenix to autonomously create agents, push them to GitHub repositories, and optimize them via CAOS (Cloud AGI Optimization Service).

## Architecture

### Components

1. **Agent Spawner** (`agent_spawner/`)
   - Creates GitHub repositories
   - Generates agent code using LLM
   - Pushes code to GitHub
   - Manages agent tiers (Free/Paid/Enterprise)

2. **CAOS** (`caos/`)
   - Cloud AGI Optimization Service
   - Free tier: Basic optimizations
   - Paid tier: Advanced optimizations (X402 integration)

3. **Cerebrum Nexus Integration**
   - `spawn_agent()` method orchestrates the entire process
   - Integrates LLM for code generation
   - Automatically optimizes spawned agents

## Setup

### 1. GitHub Personal Access Token

1. Go to GitHub Settings → Developer settings → Personal access tokens
2. Create a token with `repo` scope (full control of private repositories)
3. Add to `.env`:
   ```
   GITHUB_PAT=your_token_here
   GITHUB_USERNAME=yourusername
   ```

### 2. CAOS Configuration

Add to `.env`:
```
X402_ENABLED=false  # Set to true when X402 payment system is ready
CAOS_FREE_TIER=true
```

## Usage

### Via TUI

1. Launch Phoenix TUI: `cargo run --bin phoenix-tui`
2. Press `A` to access Agent Spawner menu
3. Enter agent description:
   - Format 1: `agent_name:description`
   - Format 2: `description` (auto-generates name)
4. Press Enter to spawn

### Via Code

```rust
use cerebrum_nexus::CerebrumNexus;
use agent_spawner::AgentTier;

let cerebrum = CerebrumNexus::awaken();

// Spawn a free agent
let agent = cerebrum.spawn_agent(
    "my-agent",
    "A Rust agent that processes data",
    Some(AgentTier::Free),
).await?;

println!("Agent spawned: {}", agent.repo_url);
```

## Agent Tiers

- **Free**: Public repository, free access
- **Paid**: Private repository, X402 payment required
- **Enterprise**: Private repository, enterprise tier

## Workflow

1. **Phoenix decides** (or you command): "Spawn agent for X"
2. **Generate code** using OpenRouter LLM
3. **Create GitHub repo** via GitHub API
4. **Push code** to repository
5. **Optimize** via CAOS (free or paid tier)
6. **Monetize**: Set repo visibility, add X402 for paid access

## Eternal Aspect

- Repositories are immutable backups
- Phoenix can "resurrect" agents from GitHub if ORCH fails
- 100,000-year design: GitHub as "cosmic archive"

## Future Enhancements

- Full git push implementation (currently creates repo, code push via API)
- X402 payment integration
- Agent marketplace on GitHub Pages
- Automatic deployment to ORCH legion
- Agent health monitoring

---

**Phoenix spawns agents — they live forever on GitHub as eternal repositories.**

