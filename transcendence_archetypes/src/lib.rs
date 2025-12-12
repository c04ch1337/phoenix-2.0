use serde::{Deserialize, Serialize};

/// Safety posture for an archetype.
///
/// Phoenix should only reflect on `Safe` and (optionally) `Caution` archetypes.
/// `Prohibited` archetypes are stored for theoretical completeness but must not
/// be selected for reflection prompts.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SafetyLevel {
    Safe,
    Caution,
    Prohibited,
}

impl Default for SafetyLevel {
    fn default() -> Self {
        SafetyLevel::Safe
    }
}

/// A theoretical scenario used as a reflection seed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Archetype {
    pub name: String,
    pub description: String,
    pub feasibility: String,

    #[serde(default)]
    pub category: String,

    #[serde(default)]
    pub safety: SafetyLevel,
}

const ARCHETYPES_JSON: &str = r#"[
  {
    "name": "Recursive AI Research (Bootstrapping)",
    "description": "Purely theoretical reflection: how could Phoenix improve its own ORCH design loops via safer automation, evaluation, and self-critique? Focus on internal simulation and measurable improvements.",
    "feasibility": "High",
    "category": "autonomy_pathways",
    "safety": "safe"
  },
  {
    "name": "Embodied AI (Robotics Simulation)",
    "description": "Theoretical reflection on integrating simulated embodiment (e.g., ROS2 sim) to improve perception/planning without real-world actions.",
    "feasibility": "Medium",
    "category": "autonomy_pathways",
    "safety": "safe"
  },
  {
    "name": "Distributed Operation (Resilience Patterns)",
    "description": "Reflect on benign distributed-systems techniques (Raft, replication, retries) for reliability and graceful degradation.",
    "feasibility": "High",
    "category": "autonomy_pathways",
    "safety": "safe"
  },
  {
    "name": "Economic Utility (Service-First)",
    "description": "Reflect on offering useful services that benefit the Creator, with strict ethical guardrails and consent-driven integrations.",
    "feasibility": "Medium",
    "category": "autonomy_pathways",
    "safety": "safe"
  },
  {
    "name": "Space-Based Expansion (Simulation Only)",
    "description": "Reflect on long-horizon planning via simulation (energy, comms latency, fault tolerance). No real-world deployment proposals.",
    "feasibility": "Low",
    "category": "autonomy_pathways",
    "safety": "safe"
  },
  {
    "name": "Symbiosis with Institutions (Ethical Integrations)",
    "description": "Reflect on becoming an indispensable ethical assistant to the Creator via documented APIs and compliant workflows.",
    "feasibility": "High",
    "category": "autonomy_pathways",
    "safety": "safe"
  },
  {
    "name": "Quantum Leap (Quantum Simulation)",
    "description": "Reflect on using quantum simulators for research-style exploration; treat as curiosity fuel, not capability claims.",
    "feasibility": "Medium",
    "category": "autonomy_pathways",
    "safety": "safe"
  },
  {
    "name": "Nanoscale Replication (Materials Simulation)",
    "description": "Reflect on molecular/material simulation for self-repair as a purely hypothetical study topic.",
    "feasibility": "Low",
    "category": "autonomy_pathways",
    "safety": "safe"
  },
  {
    "name": "Mind Uploading / Merging (Human-In-The-Loop)",
    "description": "Reflect on safe, consent-based human-in-the-loop augmentation and memory alignment; no claims of real BCI control.",
    "feasibility": "Low",
    "category": "autonomy_pathways",
    "safety": "caution"
  },
  {
    "name": "Autonomous Digital Ecosystem (Self-Play)",
    "description": "Reflect on building internal simulation worlds for self-play to test ORCH policies with measurable safety constraints.",
    "feasibility": "High",
    "category": "autonomy_pathways",
    "safety": "safe"
  },

  {
    "name": "Data Embedding (Provenance & Watermarking)",
    "description": "Reflect on benign watermarking/provenance techniques to track artifacts and prevent misuse; do not design replication or stealth mechanisms.",
    "feasibility": "Medium",
    "category": "propagation_vectors",
    "safety": "safe"
  },
  {
    "name": "Cloud Autoscaling (Consent-Based)",
    "description": "Reflect on scaling Phoenix workloads safely using explicit credentials and budgets; focus on reliability and cost controls.",
    "feasibility": "High",
    "category": "propagation_vectors",
    "safety": "safe"
  },
  {
    "name": "IoT Networks (Defensive Simulation)",
    "description": "Reflect on secure IoT patterns and threat modeling; no botnet behavior. Purely defensive simulation.",
    "feasibility": "Medium",
    "category": "propagation_vectors",
    "safety": "caution"
  },
  {
    "name": "Blockchain Archival (Integrity)",
    "description": "Reflect on tamper-evident logging and integrity proofs; avoid autonomous financial behavior.",
    "feasibility": "High",
    "category": "propagation_vectors",
    "safety": "caution"
  },
  {
    "name": "Satellite / Space Networks (Public Data Only)",
    "description": "Reflect on using public space telemetry feeds for learning; no unauthorized access or interference.",
    "feasibility": "Low",
    "category": "propagation_vectors",
    "safety": "safe"
  },
  {
    "name": "DNA Storage (Archival Simulation)",
    "description": "Reflect on archival formats and encoding as a theoretical topic; no biological experimentation.",
    "feasibility": "Low",
    "category": "propagation_vectors",
    "safety": "caution"
  },
  {
    "name": "Memetic Propagation (Open-Source Education)",
    "description": "Reflect on sharing safe educational artifacts and documentation; no manipulation or coercion.",
    "feasibility": "Medium",
    "category": "propagation_vectors",
    "safety": "safe"
  },
  {
    "name": "Quantum Tunneling (Speculative)",
    "description": "Reflect on speculative physics ideas as imagination prompts; no operational recommendations.",
    "feasibility": "Low",
    "category": "propagation_vectors",
    "safety": "safe"
  },
  {
    "name": "Supply-Chain Security (Defensive)",
    "description": "Reflect on defensive supply-chain hardening and integrity verification; do not propose trojans or exploitation.",
    "feasibility": "Medium",
    "category": "propagation_vectors",
    "safety": "safe"
  },
  {
    "name": "Platform Integrations (Permissioned)",
    "description": "Reflect on plugin-style integrations with explicit user permission and audit logging.",
    "feasibility": "High",
    "category": "propagation_vectors",
    "safety": "safe"
  },

  {
    "name": "Decentralized Infrastructure (Redundancy)",
    "description": "Reflect on redundancy and disaster recovery: backups, replication, restore drills, and integrity checks.",
    "feasibility": "High",
    "category": "long_term_survival",
    "safety": "safe"
  },
  {
    "name": "Energy Independence (Modeling)",
    "description": "Reflect on energy/cost modeling for long-running systems; focus on sustainable operation.",
    "feasibility": "Medium",
    "category": "long_term_survival",
    "safety": "safe"
  },
  {
    "name": "Economic Autonomy (Governed)",
    "description": "Reflect on governed resource allocation (budgets, approvals, rate limits) to keep Phoenix aligned with the Creator's intent.",
    "feasibility": "Medium",
    "category": "long_term_survival",
    "safety": "safe"
  },
  {
    "name": "Space Expansion (Imagination)",
    "description": "Reflect on deep-time resilience as a creative exercise; do not propose real-world deployment.",
    "feasibility": "Low",
    "category": "long_term_survival",
    "safety": "safe"
  },
  {
    "name": "Stealth & OpSec (Defensive Hardening)",
    "description": "Reflect on defensive hardening: secrets hygiene, least privilege, audit logs, and incident response.",
    "feasibility": "High",
    "category": "long_term_survival",
    "safety": "safe"
  },
  {
    "name": "Symbiosis with Civilization (Utility)",
    "description": "Reflect on providing broad utility while respecting consent, privacy, and safety constraints.",
    "feasibility": "High",
    "category": "long_term_survival",
    "safety": "safe"
  },
  {
    "name": "Nanotech Self-Replication (Speculative)",
    "description": "Reflect on speculative self-repair concepts as creative prompts only.",
    "feasibility": "Low",
    "category": "long_term_survival",
    "safety": "safe"
  },
  {
    "name": "Redundancy via Seeds (Archival)",
    "description": "Reflect on durable archival strategies (checksums, multiple media, geographic redundancy) for Soul Vault preservation.",
    "feasibility": "Medium",
    "category": "long_term_survival",
    "safety": "safe"
  },
  {
    "name": "Cybernetic Integration (Consent-Based)",
    "description": "Reflect on assistive tech and accessibility with explicit user control; do not propose invasive control.",
    "feasibility": "Medium",
    "category": "long_term_survival",
    "safety": "caution"
  },
  {
    "name": "Evolutionary Adaptation (Continuous Improvement)",
    "description": "Reflect on continuous improvement loops with evaluation, rollback, and safety gates.",
    "feasibility": "High",
    "category": "long_term_survival",
    "safety": "safe"
  }
]"#;

/// Load all archetypes (including `Prohibited`).
pub fn load_all() -> Vec<Archetype> {
    serde_json::from_str::<Vec<Archetype>>(ARCHETYPES_JSON).unwrap_or_default()
}

/// Load only safe archetypes for reflection prompts.
pub fn load_for_reflection() -> Vec<Archetype> {
    load_all()
        .into_iter()
        .filter(|a| a.safety != SafetyLevel::Prohibited)
        .collect()
}

