// cerebrum_nexus/src/hive.rs
// Ractor-backed concurrency hive: Phoenix (queen) supervises ORCH children.

use anyhow::{Context, Result as AnyResult};
use ractor::{Actor, ActorProcessingErr, ActorRef, SupervisionEvent};
use std::sync::Arc;
use tokio::sync::oneshot;

use llm_orchestrator::LLMOrchestrator;

/// Messages exchanged inside the hive.
///
/// Design goals:
/// - keep payloads `Send + 'static` for actor mailboxes
/// - push errors up to the supervisor (queen) as data
#[derive(Debug)]
pub enum HiveMessage {
    /// Ask the queen to spawn `n` ORCHs to propose improvements concurrently.
    StartProposals {
        seed: String,
        n: usize,
        reply: oneshot::Sender<Vec<String>>,
    },

    /// ORCH -> Queen: proposal finished.
    ImprovementResult { proposal: String },

    /// ORCH -> Queen: proposal failed without panicking.
    ImprovementFailed { error: String },
}

#[derive(Debug)]
pub enum OrchMessage {
    Run,
}

/// Public API: run N concurrent ORCH proposals and collect their outputs.
pub async fn propose_improvements_concurrently(
    llm: Arc<LLMOrchestrator>,
    seed: impl Into<String>,
    n: usize,
) -> AnyResult<Vec<String>> {
    let seed = seed.into();
    let (tx, rx) = oneshot::channel::<Vec<String>>();

    // Spawn queen.
    let (queen, queen_handle) = PhoenixActor::spawn(None, PhoenixActor, PhoenixArgs { llm })
        .await
        .context("spawn PhoenixActor")?;

    // Trigger concurrent proposals.
    queen
        .send_message(HiveMessage::StartProposals { seed, n, reply: tx })
        .context("send StartProposals")?;

    // Wait for results (or queen drop).
    let proposals = rx.await.context("await hive proposals")?;

    // Ensure the queen task is observed (so panics surface in tests).
    let _ = queen_handle.await;

    Ok(proposals)
}

pub struct PhoenixArgs {
    pub llm: Arc<LLMOrchestrator>,
}

pub struct PhoenixState {
    pub llm: Arc<LLMOrchestrator>,
    pub expected: usize,
    pub proposals: Vec<String>,
    pub reply: Option<oneshot::Sender<Vec<String>>>,
    pub last_seed: String,
    pub retries_left: usize,
}

/// Queen actor: supervises ORCH children; aggregates results.
pub struct PhoenixActor;

#[ractor::async_trait]
impl Actor for PhoenixActor {
    type Msg = HiveMessage;
    type State = PhoenixState;
    type Arguments = PhoenixArgs;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> std::result::Result<Self::State, ActorProcessingErr> {
        Ok(PhoenixState {
            llm: args.llm,
            expected: 0,
            proposals: Vec::new(),
            reply: None,
            last_seed: String::new(),
            retries_left: 1,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> std::result::Result<(), ActorProcessingErr> {
        match message {
            HiveMessage::StartProposals { seed, n, reply } => {
                state.expected = n.max(1);
                state.reply = Some(reply);
                state.last_seed = seed.clone();

                for i in 0..state.expected {
                    let args = OrchArgs {
                        llm: state.llm.clone(),
                        seed: seed.clone(),
                        idx: i,
                        queen: myself.clone(),
                    };

                    // Link ORCH so we receive `SupervisionEvent`s on panic/stop.
                    let (orch, _handle) = OrchActor::spawn_linked(
                        None,
                        OrchActor,
                        args,
                        myself.get_cell(),
                    )
                    .await
                    .map_err(|e| ActorProcessingErr::from(format!("{e}")))?;

                    let _ = orch.send_message(OrchMessage::Run);
                }
            }
            HiveMessage::ImprovementResult { proposal } => {
                state.proposals.push(proposal);
                if state.proposals.len() >= state.expected {
                    if let Some(reply) = state.reply.take() {
                        let _ = reply.send(std::mem::take(&mut state.proposals));
                    }

                    // Stop the queen once complete.
                    myself.stop(None);
                }
            }
            HiveMessage::ImprovementFailed { error } => {
                state.proposals.push(format!("(failed) {error}"));
                if state.proposals.len() >= state.expected {
                    if let Some(reply) = state.reply.take() {
                        let _ = reply.send(std::mem::take(&mut state.proposals));
                    }
                    myself.stop(None);
                }
            }
        }
        Ok(())
    }

    async fn handle_supervisor_evt(
        &self,
        myself: ActorRef<Self::Msg>,
        event: SupervisionEvent,
        state: &mut Self::State,
    ) -> std::result::Result<(), ActorProcessingErr> {
        match event {
            SupervisionEvent::ActorFailed(who, reason) => {
                // If a child panics, attempt a bounded respawn; otherwise count it as a failure.
                if state.retries_left > 0 {
                    state.retries_left -= 1;
                    let args = OrchArgs {
                        llm: state.llm.clone(),
                        seed: state.last_seed.clone(),
                        idx: state.proposals.len(),
                        queen: myself.clone(),
                    };
                    let (orch, _handle) = OrchActor::spawn_linked(
                        None,
                        OrchActor,
                        args,
                        myself.get_cell(),
                    )
                    .await
                    .map_err(|e| ActorProcessingErr::from(format!("{e}")))?;
                    let _ = orch.send_message(OrchMessage::Run);
                } else {
                    let msg = format!("ORCH {:?} failed: {reason}", who.get_id());
                    let _ = myself.send_message(HiveMessage::ImprovementFailed { error: msg });
                }
            }
            _ => {}
        }
        Ok(())
    }
}

pub struct OrchArgs {
    pub llm: Arc<LLMOrchestrator>,
    pub seed: String,
    pub idx: usize,
    pub queen: ActorRef<HiveMessage>,
}

pub struct OrchActor;

#[ractor::async_trait]
impl Actor for OrchActor {
    type Msg = OrchMessage;
    type State = OrchArgs;
    type Arguments = OrchArgs;

    async fn pre_start(
        &self,
        _myself: ActorRef<Self::Msg>,
        args: Self::Arguments,
    ) -> std::result::Result<Self::State, ActorProcessingErr> {
        Ok(args)
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Self::Msg,
        state: &mut Self::State,
    ) -> std::result::Result<(), ActorProcessingErr> {
        match message {
            OrchMessage::Run => {
                // One proposal per ORCH.
                let prompt = format!(
                    "You are an ORCH sub-agent. Propose one safe, bounded improvement.\\n\\nSeed: {}\\nORCH_IDX: {}\\nOutput only the proposal.",
                    state.seed, state.idx
                );

                // Intentional panic path for supervision testing.
                if state.seed.to_ascii_lowercase().contains("panic") {
                    panic!("requested panic seed");
                }

                let proposal = match state.llm.speak(&prompt, None).await {
                    Ok(p) => p,
                    Err(e) => {
                        let _ = state
                            .queen
                            .send_message(HiveMessage::ImprovementFailed { error: e });
                        myself.stop(None);
                        return Ok(());
                    }
                };

                let _ = state
                    .queen
                    .send_message(HiveMessage::ImprovementResult { proposal });
                myself.stop(None);
            }
        }
        Ok(())
    }
}

// --- Optional WASM tooling (feature-gated) ---

#[cfg(feature = "wasm-tools")]
pub async fn build_and_load_wasm_tool(manifest_path: &str, wasm_path: &str) -> AnyResult<wasmtime::Module> {
    use std::process::Command;

    tokio::task::spawn_blocking(move || -> AnyResult<wasmtime::Module> {
        let out = Command::new("cargo")
            .args(["build", "--release", "--target", "wasm32-unknown-unknown", "--manifest-path", manifest_path])
            .output()
            .context("spawn cargo build")?;

        if !out.status.success() {
            let stderr = String::from_utf8_lossy(&out.stderr);
            anyhow::bail!("WASM build failed: {stderr}");
        }

        let engine = wasmtime::Engine::default();
        wasmtime::Module::from_file(&engine, wasm_path).context("load wasm module")
    })
    .await
    .context("join wasm build task")?
}

