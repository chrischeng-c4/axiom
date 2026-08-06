"""Black-box contract proving Python Spec health never routes through the
retired legacy Markdown CB-replay path.

Drives the real, compiled `aw health` command against `guard` -- a genuine
Python Spec (`PythonV1`) project already live in this repository's own
`aw.toml` -- rather than a synthetic fixture, because the promise under test
is specifically about *this* artifact model's interaction with the
legacy-replay gate, and `guard` already exercises the real registry-resolved
`effective_artifact_model()` path with zero fixture construction risk.

The oracle is the exact advisory-note text the legacy-replay gate
(`apps/agentic-workflow/src/cli/project.rs::legacy_health_replay_enabled`)
emits in place of actually running the retired Markdown force-regeneration
replay, cross-checked in both its default (flags absent) form and its
strongest form: passing `--verify-traceability` / `--verify-cb` explicitly
asks the CLI to run the legacy replay, and the case proves the Python Spec
gate still overrides that explicit request down to the same advisory note
rather than ever attempting the retired path.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import REPOSITORY_ROOT, final_json, run_aw

CASE_ID = "aw-core-client-python-spec-health-bypasses-legacy-cb-replay"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "python-spec-health-bypasses-legacy-cb-replay"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-python-spec-health-bypasses-legacy-cb-replay"
)
ASSERTIONS = (
    "`aw health --project guard traceability` against the real, live "
    "`guard` Python Spec project reports `status`/`assessment` `done`"
    "/`healthy` (not blocked or errored) with `traceability_evaluated: "
    "false` and the exact advisory note 'legacy source-marker "
    "traceability is advisory for Python Spec project `guard`; EC/TD "
    "semantic health is authoritative' -- the legacy source-marker path "
    "is skipped and surfaced only as non-fatal advisory text",
    "`aw health --project guard cb` on the same real project reports the "
    "same healthy/done envelope with `cb_verify_evaluated: false`, "
    "`cb_verify_clean: true`, and the exact advisory note 'legacy "
    "Markdown CB replay is advisory for Python Spec project `guard`; "
    "Python artifact readiness is authoritative' -- the retired Markdown "
    "force-regeneration replay this capability names is never invoked, "
    "and its absence is not treated as a failure",
    "passing `--verify-traceability` / `--verify-cb` explicitly -- an "
    "operator request that would run the legacy replay for a real Legacy"
    "-model project -- still leaves both `traceability_evaluated` and "
    "`cb_verify_evaluated` `false` with byte-identical advisory notes on "
    "`guard`, proving the bypass is a genuine artifact-model gate rather "
    "than merely 'the flag was never passed'",
)

_TRACEABILITY_NOTE = (
    "legacy source-marker traceability is advisory for Python Spec "
    "project `guard`; EC/TD semantic health is authoritative"
)
_CB_NOTE = (
    "legacy Markdown CB replay is advisory for Python Spec project "
    "`guard`; Python artifact readiness is authoritative"
)


def _assert_healthy_envelope(payload: dict, section: str) -> None:
    assert payload["status"] == "done", payload
    assert payload["assessment"] == "healthy", payload
    assert payload["section"] == section, payload
    assert payload["project"] == "guard", payload


def verify() -> list[str]:
    # -- phase 1: default traceability section, no verify flag ------------
    traceability = final_json(
        run_aw(REPOSITORY_ROOT, "health", "--project", "guard", "traceability")
    )
    _assert_healthy_envelope(traceability, "traceability")
    assert traceability["data"]["traceability_evaluated"] is False, traceability
    assert traceability["data"]["traceability_note"] == _TRACEABILITY_NOTE, traceability

    # -- phase 2: default cb section, no verify flag -----------------------
    cb = final_json(run_aw(REPOSITORY_ROOT, "health", "--project", "guard", "cb"))
    _assert_healthy_envelope(cb, "cb")
    assert cb["data"]["cb_verify_evaluated"] is False, cb
    assert cb["data"]["cb_verify_clean"] is True, cb
    assert cb["data"]["cb_verify_note"] == _CB_NOTE, cb

    # -- phase 3: explicit --verify-traceability / --verify-cb requests the
    #    legacy replay; the Python Spec gate must still override it -------
    forced_traceability = final_json(
        run_aw(
            REPOSITORY_ROOT,
            "health",
            "--project",
            "guard",
            "--verify-traceability",
            "traceability",
        )
    )
    _assert_healthy_envelope(forced_traceability, "traceability")
    assert forced_traceability["data"]["traceability_evaluated"] is False, forced_traceability
    assert forced_traceability["data"]["traceability_note"] == _TRACEABILITY_NOTE, (
        forced_traceability
    )

    forced_cb = final_json(
        run_aw(REPOSITORY_ROOT, "health", "--project", "guard", "--verify-cb", "cb")
    )
    _assert_healthy_envelope(forced_cb, "cb")
    assert forced_cb["data"]["cb_verify_evaluated"] is False, forced_cb
    assert forced_cb["data"]["cb_verify_note"] == _CB_NOTE, forced_cb

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
