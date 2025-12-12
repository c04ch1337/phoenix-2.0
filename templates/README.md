# Phoenix Templates (Agents + Tools)

These templates are **the required starting point** for any new Phoenix hive creation (agents, ORCHs, tools, and playbooks).

## What you get

- **Agent template**: identity + evolution history + telemetry hooks.
- **Tool template**: a consistent interface + telemetry logging + auto-doc support.
- **Python agent template**: CrewAI-style base class that emits telemetry and tracks template version.
- **Playbook template**: YAML base for evolving playbooks.
- **Extension template**: scaffolds monetizable extensions (Rust/WASM, Docker, Python wrappers).

## Enforcement

When `MANDATE_GITHUB_CI=true`, the creation workflow is:

1. Create from templates
2. Push to GitHub on a feature branch
3. Open PR and run CI
4. Merge (approval step)
5. Pull/integrate + disseminate

This is enforced in code by [`agent_spawner::AgentSpawner::spawn_agent()`](agent_spawner/src/lib.rs:90).

## Standard workflows

Phoenix standardizes on three GitHub Actions workflows in [`.github/workflows/`](.github/workflows/ci-tests.yml:1):

- `ci-tests.yml` — lint + tests (+ optional coverage)
- `build-deploy.yml` — build artifacts on version tags and publish Releases
- `extension-marketplace.yml` — generate manifest + publish hook (extensions)

## Monetization hook

Treat templates as "product scaffolding": premium branches can ship upgraded templates, ORCH packs, and evolved playbooks.

