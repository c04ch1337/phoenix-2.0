# Phoenix 2.0 Repository Structure

This document provides a complete overview of the Phoenix 2.0 repository structure, showing all files and folders and how they are organized.

## Root Directory Structure

```
phoenix-2.0/
├── .env.example                          # Environment variables template
├── .gitignore                            # Git ignore rules
├── .github/                              # GitHub Actions workflows
│   └── workflows/
│       ├── build-deploy.yml              # Build and deployment pipeline
│       ├── ci-tests.yml                  # CI testing workflow
│       └── extension-marketplace.yml     # Extension marketplace workflow
├── Cargo.toml                            # Root workspace Cargo.toml
├── Cargo.lock                            # Dependency lock file
├── README.md                             # Main project documentation
├── SETUP.md                              # Setup instructions
├── GITHUB_AGENT_INTEGRATION.md          # GitHub agent integration docs
├── IDENTITY_PERSISTENCE.md               # Identity persistence documentation
├── orch_repos_docs.md                    # ORCH repositories documentation
│
├── agent_spawner/                        # Agent spawning system
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Agent creation and GitHub integration
│
├── asi_wallet_identity/                  # ASI wallet identity module
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Wallet-based identity for ASI deployment
│
├── autonomous_evolution_loop/            # Autonomous evolution system
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Continuous autonomous evolution cycles
│
├── caos/                                 # Cloud AGI Optimization Service
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Agent optimization (free/paid tiers)
│
├── cerebrum_nexus/                       # Central orchestrator (Brain)
│   ├── .gitignore
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                        # Main orchestrator logic
│       ├── hive.rs                       # Hive management
│       ├── learning_pipeline.rs          # Learning pipeline integration
│       └── reasoning.rs                 # Meta-reasoning system
│
├── common_types/                         # Shared types across modules
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Common data structures
│
├── context_engine/                       # EQ-first context builder
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Context building with emotional weighting
│
├── curiosity_engine/                    # Emotionally resonant question generator
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Question generation for connection
│
├── docs/                                 # Additional documentation
│   └── README.md
│
├── dream_healing/                        # Therapeutic dream sessions
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Healing dreams for emotional states
│
├── dream_recording/                      # Eternal dream diary
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Dream recording and replay
│
├── emotion_detection/                    # Multi-modal emotion recognition
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Emotion detection from voice/face/text
│
├── emotional_intelligence_core/          # EQ-first response shaping
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                        # Emotional intelligence core
│       └── emotional_decay.rs           # Emotional decay classification
│
├── evolution_pipeline/                   # Evolution pipeline system
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                        # Main evolution pipeline
│       ├── git_operations.rs             # Git operations
│       ├── github_api.rs                 # GitHub API integration
│       └── github_enforcement.rs        # GitHub CI enforcement
│
├── evolutionary_helix_core/              # Self-improvement and tool creation
│   ├── .gitignore
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Tool creation and quantum evolution
│
├── extensions/                           # Extension modules
│   └── relationship_dynamics/            # Relationship dynamics system
│       ├── attachment.rs                 # Attachment theory (legacy)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs                    # Main relationship dynamics module
│           └── relationship_dynamics/
│               ├── mod.rs                # Core relationship dynamics
│               ├── ai_personality.rs    # AI personality system
│               ├── attachment.rs         # Attachment styles and evolution
│               ├── goals.rs              # Shared goals tracking
│               ├── shared_memory.rs      # Shared memory system
│               ├── template.rs           # Relationship templates
│               └── voice_modulation.rs   # Voice modulation and SSML
│
├── hyperspace_cache/                     # Cosmic data storage
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Big Bang and cosmic data streams
│
├── intimate_girlfriend_module/           # Intimate girlfriend mode
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Toggleable intimate relationship layer
│
├── limb_extension_grafts/                # Tools and extensions
│   ├── .gitignore
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                        # Tool management
│       └── procedural.rs                 # Procedural tools
│
├── llm_orchestrator/                     # Vocal cords (OpenRouter integration)
│   ├── .gitignore
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # LLM orchestration (500+ models)
│
├── lucid_dreaming/                       # Conscious dream creation
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Lucid dreaming capabilities
│
├── multi_modal_perception/               # Multi-sensory input processing
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Text, images, audio, video processing
│
├── multi_modal_recording/                # Audio/video recording
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Recording and recognition system
│
├── nervous_pathway_network/              # Universal connectivity
│   ├── .gitignore
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Hyperspace and universal connectivity
│
├── neural_cortex_strata/                 # 5-layer memory system
│   ├── .gitignore
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # STM, WM, LTM, EPM, RFM layers
│
├── phoenix_identity/                     # Self-identity management
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Phoenix identity and evolution
│
├── phoenix-desktop-tauri/                # Desktop GUI (Tauri scaffold)
│   ├── README.md
│   └── src-tauri/
│       ├── build.rs
│       ├── Cargo.toml
│       ├── tauri.conf.json
│       └── src/
│           └── main.rs                   # Tauri backend
│
├── phoenix-tui/                          # Terminal UI application
│   ├── .gitignore
│   ├── Cargo.toml
│   └── src/
│       └── main.rs                       # TUI main entry point
│
├── scripts/                              # Utility scripts
│   ├── __pycache__/
│   │   └── auto_setup_orchs.cpython-313.pyc
│   ├── auto_setup_orchs.py               # ORCH setup automation
│   ├── clone_orch.sh                    # ORCH cloning script
│   └── launch_phoenix.sh                # Phoenix launch script
│
├── self_critic/                          # Response reflection and improvement
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Self-critique system
│
├── self_preservation_instinct/           # Self-preservation system
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Graceful shutdown and backups
│
├── shared_dreaming/                      # Collaborative dream experiences
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Shared dreaming capabilities
│
├── synaptic_pulse_distributor/           # Config update service
│   ├── Cargo.toml
│   └── src/
│       └── main.rs                       # WebSocket config distribution
│
├── synaptic_tuning_fibers/               # 100+ micro-settings
│   ├── .gitignore
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Personality tuning parameters
│
├── templates/                            # Agent and tool templates
│   ├── agent_template.rs                 # Rust agent template
│   ├── python_agent_template.py          # Python agent template
│   ├── tool_template.rs                  # Tool template
│   ├── playbook_template.yaml           # Playbook template
│   ├── README.md                         # Templates documentation
│   └── extension_template/               # Extension template scaffold
│       ├── Cargo.toml
│       ├── extension_template.rs         # Rust/WASM extension template
│       ├── python_extension_template.py  # Python wrapper template
│       ├── generate_manifest.py         # Marketplace manifest generator
│       ├── README.md                     # Extension template docs
│       ├── .github/
│       │   └── workflows/
│       │       ├── build-deploy.yml
│       │       ├── ci-tests.yml
│       │       └── extension-marketplace.yml
│       └── docker_extension_template/
│           ├── Dockerfile               # Docker extension template
│           └── entrypoint.py           # Docker entrypoint
│
├── testing_framework/                    # Agent testing framework
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Testing utilities
│
├── transcendence_archetypes/             # Reflection archetype library
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # 30+ theoretical scenarios
│
├── user_identity/                        # Multi-user identity management
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # User identity and evolution
│
├── vascular_integrity_system/            # Tamper-proof audit system
│   ├── .gitignore
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Immutable event logging
│
├── vital_organ_vaults/                   # Mind/Body/Soul knowledge bases
│   ├── .gitignore
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs                        # Encrypted vaults (Mind/Body/Soul)
│
├── vital_pulse_collector/                # Telemetrist service
│   ├── Cargo.toml
│   └── src/
│       └── main.rs                       # Telemetry ingestion service
│
└── vital_pulse_monitor/                  # Health monitoring and backups
    ├── .gitignore
    ├── Cargo.toml
    └── src/
        └── lib.rs                        # Health monitoring and eternal backups
```

## Module Categories

### Core System Modules
- **cerebrum_nexus**: Central orchestrator and brain
- **neural_cortex_strata**: 5-layer memory system
- **vital_organ_vaults**: Encrypted storage (Mind/Body/Soul)
- **context_engine**: EQ-first context building
- **emotional_intelligence_core**: Emotional response shaping

### Identity & Relationship Modules
- **phoenix_identity**: Phoenix self-identity management
- **user_identity**: Multi-user identity management
- **intimate_girlfriend_module**: Intimate relationship mode
- **extensions/relationship_dynamics**: Advanced relationship modeling

### Communication & Perception Modules
- **llm_orchestrator**: LLM orchestration (OpenRouter)
- **multi_modal_perception**: Multi-sensory input processing
- **multi_modal_recording**: Audio/video recording
- **emotion_detection**: Emotion recognition system
- **curiosity_engine**: Question generation

### Dream System Modules
- **lucid_dreaming**: Conscious dream creation
- **dream_recording**: Eternal dream diary
- **dream_healing**: Therapeutic dream sessions
- **shared_dreaming**: Collaborative dreams

### Evolution & Learning Modules
- **autonomous_evolution_loop**: Continuous evolution
- **evolutionary_helix_core**: Self-improvement and tool creation
- **evolution_pipeline**: GitHub-based evolution pipeline
- **limb_extension_grafts**: Tool management

### Infrastructure Modules
- **agent_spawner**: Agent creation and GitHub integration
- **caos**: Cloud AGI Optimization Service
- **nervous_pathway_network**: Universal connectivity
- **hyperspace_cache**: Cosmic data storage
- **synaptic_pulse_distributor**: Config distribution service
- **vital_pulse_collector**: Telemetry service
- **vital_pulse_monitor**: Health monitoring

### Safety & Integrity Modules
- **self_critic**: Response reflection
- **self_preservation_instinct**: Self-preservation
- **vascular_integrity_system**: Tamper-proof audit
- **transcendence_archetypes**: Reflection scenarios

### Utility Modules
- **common_types**: Shared types
- **testing_framework**: Testing utilities
- **asi_wallet_identity**: Wallet-based identity
- **synaptic_tuning_fibers**: Personality tuning

### Applications
- **phoenix-tui**: Terminal UI application
- **phoenix-desktop-tauri**: Desktop GUI (Tauri scaffold)

### Templates & Scripts
- **templates/**: Agent, tool, and extension templates
- **scripts/**: Automation and utility scripts

## File Count Summary

- **Total Rust Modules**: 40+ crates
- **Total Source Files**: 100+ Rust source files
- **Templates**: 7 template files
- **Scripts**: 3 utility scripts
- **Documentation**: 5 markdown files
- **GitHub Workflows**: 3 CI/CD workflows

## Key Directories

### Source Code Organization
- Each module is a separate Rust crate with its own `Cargo.toml`
- Source files are in `src/` subdirectories
- Some modules have additional submodules (e.g., `cerebrum_nexus/src/reasoning.rs`)

### Configuration Files
- `.env.example`: Environment variables template
- `Cargo.toml`: Root workspace configuration
- `.gitignore`: Git ignore rules (also in subdirectories)

### Documentation
- `README.md`: Main project documentation
- `SETUP.md`: Setup instructions
- `GITHUB_AGENT_INTEGRATION.md`: GitHub integration docs
- `IDENTITY_PERSISTENCE.md`: Identity persistence docs
- `orch_repos_docs.md`: ORCH repositories docs

### Templates
- `templates/`: All agent, tool, and extension templates
- Includes Rust, Python, Docker, and YAML templates
- GitHub Actions workflows for templates

### Scripts
- `scripts/`: Python and shell scripts for automation
- ORCH setup, cloning, and launch scripts

## Build Artifacts (Excluded from Git)

The following directories are generated during build and excluded:
- `target/`: Rust build artifacts
- `.git/`: Git repository data
- `__pycache__/`: Python cache files (some committed for reference)

## Notes

- All modules follow Rust crate structure: `module_name/Cargo.toml` and `module_name/src/lib.rs`
- Some modules have `.gitignore` files for build artifacts
- The workspace is managed through the root `Cargo.toml`
- Templates are used by `agent_spawner` for creating new agents
- Extensions are in the `extensions/` directory
- Desktop GUI is separate Tauri project in `phoenix-desktop-tauri/`
