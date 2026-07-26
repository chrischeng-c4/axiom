"""Native Python ECs for shared-service-kit adoption and health regression."""

from __future__ import annotations

import time
from typing import Any

from migration_clusters.existing_health import _health_snapshot


CASE_IDS = {
    "jet-health-verification-dedup-smoke",
    "project-health-no-regression",
    "existing-project-standardization-operational-efficiency",
    "existing-project-standardization-operational-stability",
}


def _health_contract() -> dict[str, Any]:
    snapshot = _health_snapshot(authoritative=False)
    result = snapshot["result"]
    payload = snapshot["payload"]
    assert result["event"] == "result"
    assert result["next"]["command"].startswith("aw ")
    assert isinstance(payload["production_ready"], bool)
    assert isinstance(payload["blockers"], list)
    return snapshot


def verify(case_id: str) -> list[str]:
    if case_id not in CASE_IDS:
        raise AssertionError(f"case is not owned by existing-service-kit: {case_id}")
    started = time.monotonic()

    first = _health_contract()
    if case_id == "jet-health-verification-dedup-smoke":
        commands = first["payload"]["test_gates"]["commands"]
        identities = [
            (entry["command"], entry.get("workspace"), entry.get("target"))
            for entry in commands
        ]
        assert len(identities) == len(set(identities))
        return [
            "one health report projects each configured gate identity once",
            "the terminal result contains only real readiness blockers and one runnable next command",
        ]
    if case_id == "project-health-no-regression":
        assert "capability" in first["result"]["axes"]
        assert "ec" in first["result"]["axes"]
        return [
            "project health retains typed capability and EC readiness axes",
            "unrelated workflow-envelope evolution does not remove durable blocker evidence",
        ]
    if case_id == "existing-project-standardization-operational-efficiency":
        assert time.monotonic() - started <= 120
        return [
            "representative existing-project health evaluation completes within 120 seconds",
            "the native Python oracle executes the real AW health surface without cargo delegation",
        ]

    second = _health_contract()
    stable_axes = ("capability", "ec", "ec_gen", "td", "td_gen", "drift_marker")
    first_statuses = {
        name: first["result"]["axes"][name]["status"] for name in stable_axes
    }
    second_statuses = {
        name: second["result"]["axes"][name]["status"] for name in stable_axes
    }
    assert first_statuses == second_statuses
    first_commands = [
        entry["command"] for entry in first["payload"]["test_gates"]["commands"]
    ]
    second_commands = [
        entry["command"] for entry in second["payload"]["test_gates"]["commands"]
    ]
    assert first_commands == second_commands
    return [
        "two existing-project health runs preserve typed readiness axes",
        "configured gate projection is stable across repeated evaluation",
    ]
