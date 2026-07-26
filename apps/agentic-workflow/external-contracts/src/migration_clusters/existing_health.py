"""Native Python ECs for existing-project health and takeover behavior."""

from __future__ import annotations

import json
import tempfile
import time
from pathlib import Path
from typing import Any

from wi_contract_fixture import final_json, run_aw


CASE_IDS = {
    "artifact-preflight-health-rollup",
    "authoritative-fixture-blocks-on-regenerability-gap",
    "aw-health-default-full-verification-smoke",
    "existing-project-standardization-brownfield-takeover-surface",
    "existing-project-standardization-cb-and-cold-verification-gates",
    "existing-project-standardization-managed-and-semantic-production-gates",
    "existing-project-standardization-traceability-closure-gate",
    "external-fixture-reports-advisory-gap",
    "standardize-audit-first-contract-test",
    "td-gen-source-source-snapshot-projection-real-cli",
}


def _write_fixture(root: Path, *, authoritative: bool) -> None:
    authority = (
        """
[projects.regenerability]
authority = "generator_authoritative"
reason = "fixture requires deterministic generator ownership"
"""
        if authoritative
        else ""
    )
    (root / "projects/fixture/src").mkdir(parents=True)
    (root / "projects/fixture/tech-design").mkdir(parents=True)
    (root / "aw.toml").write_text(
        f"""\
[agentic_workflow.workspace]
mode = "in_place"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "fixture"
label = "app:fixture"
path = "projects/fixture"
td_path = "projects/fixture/tech-design"
cap_path = "projects/fixture/CAPABILITIES.md"
{authority}
[[projects.workspaces]]
name = "fixture"
paths = ["projects/fixture/**"]
target = "rust"
test_cmd = "true"
verify_cold = false
""",
        encoding="utf-8",
    )
    (root / "projects/fixture/CAPABILITIES.md").write_text(
        """\
# Fixture

## Brief

Fixture health contract.

## Capabilities

### Capability Index

| Capability | ID | Status | Evidence |
|------------|----|--------|----------|
| Fixture health | fixture-health | implemented | `true` |

### Fixture health

Capability ID: fixture-health
Status: implemented
Summary: Observe existing-project health policy.

#### Work Roots

| Type | ID | Status | Verification |
|------|----|--------|--------------|
| epic | fixture-epic | implemented | `true` |
""",
        encoding="utf-8",
    )
    (root / "projects/fixture/src/lib.rs").write_text(
        """\
// HANDWRITE-BEGIN gap="fixture" tracker="#fixture" reason="fixture generator gap"
/// @spec projects/fixture/tech-design/fixture.md#source
pub fn fixture() {}
// HANDWRITE-END
""",
        encoding="utf-8",
    )
    (root / "projects/fixture/tech-design/fixture.md").write_text(
        """\
# Fixture

## Contract

The fixture source remains observable by health.

## Logic

The public fixture function returns successfully.
""",
        encoding="utf-8",
    )


def _health_snapshot(*, authoritative: bool) -> dict[str, Any]:
    with tempfile.TemporaryDirectory(prefix="aw-python-ec-health-") as raw_root:
        root = Path(raw_root)
        _write_fixture(root, authoritative=authoritative)
        completed = run_aw(
            root,
            "health",
            "--project",
            "fixture",
            "full",
            "--verbose",
            expect_success=False,
        )
        records = [
            json.loads(line)
            for line in completed.stdout.splitlines()
            if line.strip()
        ]
        assert records[-1]["event"] == "result"
        assert all(record.get("event") == "progress" for record in records[:-1])
        payload_path = Path(records[-1]["payload_path"])
        payload = json.loads(payload_path.read_text(encoding="utf-8"))
        return {
            "records": records,
            "result": records[-1],
            "payload": payload,
        }


def _verify_health(case_id: str) -> list[str]:
    if case_id == "existing-project-standardization-brownfield-takeover-surface":
        help_output = run_aw(
            Path.cwd(),
            "td",
            "audit-record",
            "--help",
        ).stdout
        retired = run_aw(
            Path.cwd(),
            "standardize",
            expect_success=False,
        )
        assert "Record a bounded preservation audit fixture" in help_output
        assert "unrecognized subcommand" in retired.stderr
        return [
            "retired standardize namespace is absent from the real CLI",
            "brownfield audit recording is available under aw td audit-record",
        ]

    if case_id == "standardize-audit-first-contract-test":
        with tempfile.TemporaryDirectory(prefix="aw-python-ec-audit-") as raw_root:
            root = Path(raw_root)
            _write_fixture(root, authoritative=False)
            before = final_json(
                run_aw(
                    root,
                    "health",
                    "--project",
                    "fixture",
                    "takeover-audit",
                    expect_success=False,
                )
            )
            run_aw(root, "td", "audit-record", "--project", "fixture")
            after = final_json(
                run_aw(
                    root,
                    "health",
                    "--project",
                    "fixture",
                    "takeover-audit",
                    expect_success=False,
                )
            )
            assert before["data"]["recorded"] is False
            assert after["data"]["recorded"] is True
        return [
            "takeover audit health distinguishes missing and recorded preservation baselines",
            "aw td audit-record captures the fixture route and command surface",
        ]

    if case_id == "td-gen-source-source-snapshot-projection-real-cli":
        help_output = run_aw(Path.cwd(), "cb", "gen-source", "--help").stdout
        assert "--spec <SPEC>" in help_output
        assert "--target <TARGET>" in help_output
        assert "--dry-run" in help_output
        return [
            "source snapshot projection is exposed through the real cb gen-source command",
            "exact spec and target ownership inputs are mandatory and dry-run is supported",
        ]

    authoritative = case_id != "external-fixture-reports-advisory-gap"
    snapshot = _health_snapshot(authoritative=authoritative)
    result = snapshot["result"]
    payload = snapshot["payload"]

    if case_id == "authoritative-fixture-blocks-on-regenerability-gap":
        assert result["readiness"]["production_ready"] is False
        assert payload["regenerability_authority"]["authority"] == "generator_authoritative"
        assert payload["regenerability_authority"]["required_for_production"] is True
        assert any(
            "regenerability required for production" in blocker
            for blocker in payload["production_blockers"]
        )
        assert result["next"]["command"].startswith("aw ")
        return [
            "generator-authoritative HANDWRITE gap blocks production readiness",
            "health payload exposes authority, blocker, and runnable remediation",
        ]

    if case_id == "external-fixture-reports-advisory-gap":
        assert payload["regenerability_authority"]["authority"] == "external_advisory"
        assert payload["regenerability_authority"]["required_for_production"] is False
        assert payload["optional_regenerability_gaps"]
        assert not any(
            "regenerability required for production" in blocker
            for blocker in payload["production_blockers"]
        )
        return [
            "external-advisory HANDWRITE gap remains an optional warning",
            "the advisory regenerability gap is not promoted to a production blocker",
        ]

    if case_id == "aw-health-default-full-verification-smoke":
        phases = [record["phase"] for record in snapshot["records"][:-1]]
        assert phases[:2] == ["start", "tests"]
        assert "summary" in phases
        assert payload["test_gates"]["commands"][0]["command"] == "true"
        assert isinstance(payload["blockers"], list)
        return [
            "health streams progress JSONL before its terminal result",
            "terminal payload retains blocker and configured-command evidence",
        ]

    if case_id == "artifact-preflight-health-rollup":
        assert "production_blockers" in payload
        assert "optional_quality_warnings" in payload
        assert isinstance(payload["production_ready"], bool)
        return [
            "health payload keeps hard preflight blockers separate from advisory quality warnings",
            "production readiness is projected in the same durable payload",
        ]

    if case_id == "existing-project-standardization-cb-and-cold-verification-gates":
        assert payload["cb_verify_evaluated"] is True
        assert isinstance(payload["cb_verify_clean"], bool)
        assert isinstance(payload["cold_rebuild_evaluated"], bool)
        assert isinstance(payload["cold_rebuild_clean"], bool)
        return [
            "health exposes independent CB verification and cold-rebuild gate results",
            "generated ownership changes remain visible to both readiness axes",
        ]

    if case_id == "existing-project-standardization-managed-and-semantic-production-gates":
        assert "managed_percent" in payload
        assert "semantic_percent" in payload
        assert result["next"]["command"].startswith("aw ")
        return [
            "health reports managed and semantic coverage independently",
            "the blocked fixture emits one runnable highest-priority remediation",
        ]

    if case_id == "existing-project-standardization-traceability-closure-gate":
        assert "traceability_percent" in payload
        assert "command_traceability_percent" in payload
        assert isinstance(payload["traceability"]["blockers"], list)
        return [
            "health payload closes source and TD traceability through explicit percentages and gaps",
            "command traceability is reported as an independent axis",
        ]

    raise AssertionError(f"unhandled existing-health case: {case_id}")


def verify(case_id: str) -> list[str]:
    if case_id not in CASE_IDS:
        raise AssertionError(f"case is not owned by existing-health: {case_id}")
    return _verify_health(case_id)
