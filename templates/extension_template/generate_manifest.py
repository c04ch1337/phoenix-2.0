#!/usr/bin/env python3
"""Generate a Phoenix Marketplace manifest.

This is intentionally dependency-free so it can run in CI.
It emits `phoenix-marketplace.json` in the repo root.
"""

from __future__ import annotations

import json
import re
from pathlib import Path


def _read_cargo_metadata(cargo_toml: Path) -> tuple[str, str]:
    txt = cargo_toml.read_text(encoding="utf-8", errors="ignore")

    # Minimal TOML-ish parse (good enough for the template).
    name = "phoenix_extension"
    version = "0.1.0"

    m = re.search(r"(?m)^name\s*=\s*\"([^\"]+)\"\s*$", txt)
    if m:
        name = m.group(1)
    m = re.search(r"(?m)^version\s*=\s*\"([^\"]+)\"\s*$", txt)
    if m:
        version = m.group(1)

    return name, version


def main() -> int:
    root = Path.cwd()
    cargo_toml = root / "Cargo.toml"
    if not cargo_toml.exists():
        raise SystemExit("Cargo.toml not found; cannot infer name/version")

    name, version = _read_cargo_metadata(cargo_toml)
    manifest = {
        "name": name,
        "version": version,
        "description": "Phoenix extension",
        "template_version": "1.0.0",
        "capabilities": [],
        "billing": {"tier": "free"},
    }

    out = root / "phoenix-marketplace.json"
    out.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f"Wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

