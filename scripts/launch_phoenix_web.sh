#!/usr/bin/env bash
set -euo pipefail

echo "Launching Phoenix AGI (PAGI) Web UI backend..."
cargo run --bin phoenix-web

