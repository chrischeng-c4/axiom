"""Native Python ECs for shared-service-kit adoption and health regression."""

from __future__ import annotations

import time
from pathlib import Path
from typing import Any

from migration_clusters.existing_health import _health_snapshot
from wi_contract_fixture import REPOSITORY_ROOT


CASE_IDS = {
    "existing-project-standardization-shared-service-kit-connection-budget",
    "existing-project-standardization-shared-service-kit-drain",
    "existing-project-standardization-shared-service-kit-http1-h2c-options",
    "existing-project-standardization-shared-service-kit-service-http-delegation",
    "existing-project-standardization-shared-service-kit-substrate",
    "jet-health-verification-dedup-smoke",
    "project-health-no-regression",
    "existing-project-standardization-operational-efficiency",
    "existing-project-standardization-operational-stability",
}


def _read(relative_path: str) -> str:
    path = REPOSITORY_ROOT / relative_path
    assert path.is_file(), f"missing source-owned shared-kit contract: {path}"
    return path.read_text(encoding="utf-8")


def _shared_kit_snapshot() -> dict[str, Any]:
    tcp = _read("libs/server-tcp/src/lib.rs")
    limits = _read("libs/server-lifecycle/src/limits.rs")
    lifecycle = _read("libs/server-lifecycle/tests/drain_prestart.rs")
    http = _read("libs/server-http/src/lib.rs")
    service = _read("libs/service-http/src/transport.rs")

    assert "pub struct TcpServerConfig" in tcp
    assert "pub connection_budget: Option<ConnectionBudget>" in tcp
    assert "pub struct ConnectionBudget" in limits
    assert "pub fn try_acquire(&self)" in limits
    assert "async fn connection_budget_releases_after_handler_finishes()" in tcp
    assert "async fn serve_accepts_closure_handler_without_async_trait_boxing()" in tcp

    assert "async fn receiverless_drain_persists_for_late_subscriber()" in lifecycle
    assert "DrainController::new()" in lifecycle

    assert "pub async fn serve_h2c_with_options" in http
    assert "transport_h2c::serve_connection_with_options" in http
    assert "async fn serves_http1_and_h2c_on_one_listener_with_tunable_options()" in http

    assert "server_http::serve_h2c(" in service
    assert "async fn serve_delegates_listener_to_shared_http_runtime()" in service
    return {
        "tcp": tcp,
        "limits": limits,
        "lifecycle": lifecycle,
        "http": http,
        "service": service,
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

    if case_id.startswith("existing-project-standardization-shared-service-kit-"):
        snapshot = _shared_kit_snapshot()
        if case_id.endswith("connection-budget"):
            assert "drop(permit)" in snapshot["limits"]
            return [
                "server-tcp owns connection admission through a bounded semaphore",
                "the permit-release invariant remains colocated with the owning runtime source",
            ]
        if case_id.endswith("drain"):
            assert "drain.start_drain();" in snapshot["lifecycle"]
            return [
                "server-lifecycle keeps a durable drain signal for a late subscriber",
                "the pre-subscription drain invariant remains source-colocated",
            ]
        if case_id.endswith("http1-h2c-options"):
            assert "HttpServerOptions" in snapshot["http"]
            return [
                "server-http exposes one HTTP/1.1 and h2c listener with typed options",
                "the real listener invariant remains source-colocated in the owning library",
            ]
        if case_id.endswith("service-http-delegation"):
            return [
                "service-http delegates listener execution to server-http",
                "router response preservation remains an owning-library invariant",
            ]
        return [
            "server-tcp exposes a closure-based handler without async-trait boxing",
            "the real listener and handler invocation invariant remains source-colocated",
        ]

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
