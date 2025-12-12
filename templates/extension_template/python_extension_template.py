"""Phoenix Extension Template (Python).

Template version: 1.0.0

This is intended as a wrapper for CrewAI tools or subprocess tools.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Dict


@dataclass
class PhoenixPythonExtension:
    name: str = "example_python_extension"
    version: str = "0.1.0"
    template_version: str = "1.0.0"
    metrics: Dict[str, float] = field(default_factory=dict)

    def init(self) -> None:
        # Hook point: setup telemetry sinks, identity tags, etc.
        self.metrics.setdefault("inits", 0.0)
        self.metrics["inits"] += 1.0

    def execute(self, input: Dict[str, Any]) -> Dict[str, Any]:
        self.metrics.setdefault("executions", 0.0)
        self.metrics["executions"] += 1.0
        return {"ok": True, "echo": input}

    def telemetry_report(self) -> Dict[str, Any]:
        return {
            "name": self.name,
            "version": self.version,
            "template_version": self.template_version,
            "metrics": self.metrics,
        }

    def self_test(self) -> bool:
        try:
            out = self.execute({"ping": True})
            return bool(out.get("ok"))
        except Exception:
            return False

