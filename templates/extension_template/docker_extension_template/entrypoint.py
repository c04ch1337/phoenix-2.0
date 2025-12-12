"""Phoenix Docker Extension Template.

Template version: 1.0.0

This is a minimal entrypoint that supports:
- a healthcheck mode
- a stdin/stdout JSON execution mode (placeholder)
"""

from __future__ import annotations

import json
import sys


def healthcheck() -> int:
    # Replace with real dependency checks.
    return 0


def main() -> int:
    if "--healthcheck" in sys.argv:
        return healthcheck()

    raw = sys.stdin.read().strip() or "{}"
    try:
        payload = json.loads(raw)
    except Exception:
        payload = {"raw": raw}

    out = {"ok": True, "template_version": "1.0.0", "echo": payload}
    sys.stdout.write(json.dumps(out))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

