"""Phoenix Python Agent Template (CrewAI-compatible-ish).

Template version: 1.0.0
"""

from __future__ import annotations

from dataclasses import dataclass, field
from time import time
from typing import Dict, List, Optional


@dataclass
class EvolutionEntry:
    ts_unix: int
    change_type: str
    reason: str


@dataclass
class PhoenixAgentBase:
    name: str
    version: str = "0.1.0"
    template_version: str = "1.0.0"
    creator: str = "Phoenix Queen"
    playbook_version: int = 1
    evolution_history: List[EvolutionEntry] = field(default_factory=list)
    telemetry: Dict[str, float] = field(default_factory=dict)

    def __post_init__(self) -> None:
        if not self.evolution_history:
            self.evolution_history.append(
                EvolutionEntry(ts_unix=int(time()), change_type="creation", reason="bootstrapped_from_template")
            )

    def record_metric(self, key: str, value: float) -> None:
        self.telemetry[key] = float(value)

    def emit_telemetry(self) -> dict:
        # Hook point: send to Phoenix telemetrist.
        return {
            "name": self.name,
            "version": self.version,
            "template_version": self.template_version,
            "telemetry": self.telemetry,
            "playbook_version": self.playbook_version,
        }

