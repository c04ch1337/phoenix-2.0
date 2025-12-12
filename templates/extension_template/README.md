# Phoenix Extension Template

This directory is a **scaffold** for building pluggable, monetizable Phoenix extensions.

## Supported packaging

- Rust/WASM-style extension interface: [`extension_template.rs`](templates/extension_template/extension_template.rs:1)
- Dockerized extension skeleton: [`Dockerfile`](templates/extension_template/docker_extension_template/Dockerfile:1)
- Python wrapper skeleton: [`python_extension_template.py`](templates/extension_template/python_extension_template.py:1)

## Marketplace manifest

The Rust template includes a `generate_manifest()` hook (JSON) intended for GitHub Marketplace-style metadata.

## Required invariants

- Must emit telemetry
- Must implement a `self_test()` sanity check
- Must declare `template_version`
- Must include GitHub Actions CI/CD workflows under `.github/workflows/`:
  - `ci-tests.yml` (mandatory)
  - `build-deploy.yml`
  - `extension-marketplace.yml` (monetization/registry hook)
- When `MANDATE_GITHUB_CI=true`, must be introduced via PR + CI

## Badges

Add a CI badge to your extension README:

`[![CI Tests](https://github.com/<OWNER>/<REPO>/actions/workflows/ci-tests.yml/badge.svg)](https://github.com/<OWNER>/<REPO>/actions/workflows/ci-tests.yml)`

