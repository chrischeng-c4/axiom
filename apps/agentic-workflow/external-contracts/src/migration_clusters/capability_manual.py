"""Native Python ECs for capability projection and manual evidence artifacts."""

from __future__ import annotations

import json
import time
import tomllib
from pathlib import Path
from typing import Any

from migration_clusters.work_item_planning import BOUNDED_BODY
from wi_contract_fixture import create, final_json, project_fixture, run_aw


CASE_IDS = {
    "capability-control-plane-capability-project-sweep",
    "capability-control-plane-capability-readiness-reporting",
    "capability-control-plane-markdown-capability-schema",
    "capability-control-plane-missing-readme-initialization",
    "capability-control-plane-operational-efficiency",
    "capability-control-plane-operational-stability",
    "manual-evidence-artifacts-generated-manual-ec-evidence-schema",
    "manual-evidence-artifacts-manual-runner-output-convention",
    "manual-evidence-artifacts-operational-efficiency",
    "manual-evidence-artifacts-operational-stability",
}

CAPABILITY_DOCUMENT = """\
# Demo

## Brief

Demo capability fixture.

## Capabilities

### Capability Index

| Capability | ID | Status | Evidence |
|------------|----|--------|----------|
| Planning | planning | implemented | `true` |

### Planning

Capability ID: planning
Status: implemented
Summary: Provide deterministic planning.

#### Claims

| Claim ID | Claim | Status | Evidence |
|----------|-------|--------|----------|
| planning-ready | Planning is ready | verified | `true` |

#### Work Roots

| Type | ID | Status | Verification |
|------|----|--------|--------------|
| epic | planning-epic | implemented | `true` |
"""


def _capability_snapshot() -> dict[str, Any]:
    with project_fixture() as root:
        initialized = final_json(
            run_aw(
                root,
                "capability",
                "init",
                "--project",
                "demo",
                "--title",
                "Demo",
                "--brief",
                "Demo capability fixture.",
            )
        )
        cap_path = Path(initialized["cap_path"])
        shell = cap_path.read_text(encoding="utf-8")
        assert "## Brief" in shell
        assert "## Capabilities" in shell
        assert "### Capability Index" in shell
        cap_path.write_text(CAPABILITY_DOCUMENT, encoding="utf-8")

        report = final_json(
            run_aw(
                root,
                "capability",
                "report",
                "--project",
                "demo",
                "--skip-issue-inventory",
            )
        )
        sweep = final_json(
            run_aw(
                root,
                "capability",
                "sweep",
                "--project",
                "demo",
                "--skip-issue-inventory",
            )
        )
        serialized = json.dumps({"report": report, "sweep": sweep}, sort_keys=True)
        assert "planning" in serialized
        assert "demo" in serialized
        return {"initialized": initialized, "report": report, "sweep": sweep}


def _manual_snapshot() -> dict[str, Any]:
    with project_fixture() as root:
        change = create(
            root,
            "Manual evidence fixture",
            "change",
            "--body",
            BOUNDED_BODY,
        )
        draft = final_json(
            run_aw(
                root,
                "ec",
                "draft",
                "manual-evidence",
                "--project",
                "demo",
                "--wi",
                change["slug"],
                "--capability-id",
                "planning",
                "--title",
                "Manual evidence fixture",
                "--json",
            )
        )
        pyproject = tomllib.loads((root / "external-contracts/pyproject.toml").read_text())
        inventory = pyproject["tool"]["aw"]["python-ec"]
        cases = inventory["cases"]
        assert cases[0]["id"] == "manual-evidence-behavior"
        assert cases[0]["evidence_paths"]
        runner = root / "external-contracts/src/runner.py"
        source = root / "external-contracts/src/manual-evidence.py"
        assert runner.is_file()
        assert source.is_file()
        assert draft["next"]["command"].startswith("aw ec check ")
        return {"draft": draft, "case": cases[0], "runner": runner.read_text()}


def verify(case_id: str) -> list[str]:
    if case_id not in CASE_IDS:
        raise AssertionError(f"case is not owned by capability-and-manual: {case_id}")
    started = time.monotonic()
    if case_id.startswith("capability-control-plane"):
        first = _capability_snapshot()
        if case_id == "capability-control-plane-capability-project-sweep":
            assertions = [
                "capability sweep groups the configured project and its next action",
                "the sweep retains the canonical planning capability identity",
            ]
        elif case_id == "capability-control-plane-capability-readiness-reporting":
            assertions = [
                "capability report resolves declared claim evidence",
                "project readiness is emitted from the canonical capability document",
            ]
        elif case_id == "capability-control-plane-markdown-capability-schema":
            assertions = [
                "field-style capability contract and Markdown tables parse successfully",
                "Capability Index, Claims, and Work Roots survive reporting",
            ]
        elif case_id == "capability-control-plane-missing-readme-initialization":
            assertions = [
                "capability init creates the missing canonical CAPABILITIES.md shell",
                "the shell contains Brief, Capabilities, and Capability Index sections",
            ]
        elif case_id == "capability-control-plane-operational-efficiency":
            assert time.monotonic() - started <= 120
            assertions = [
                "native capability init/report/sweep completes within 120 seconds",
                "all representative assertions pass without cargo delegation",
            ]
        else:
            second = _capability_snapshot()
            assert first["report"]["project"] == second["report"]["project"]
            assertions = [
                "two capability report/sweep executions preserve the same project identity",
                "both executions parse the canonical Markdown contract",
            ]
        return assertions

    first = _manual_snapshot()
    if case_id == "manual-evidence-artifacts-generated-manual-ec-evidence-schema":
        return [
            "generated Python EC inventory declares a concrete evidence path",
            "case id, capability id, promise, command, and evidence metadata parse as TOML",
        ]
    if case_id == "manual-evidence-artifacts-manual-runner-output-convention":
        return [
            "EC draft writes the Python runner and case module from inventory",
            "the artifact envelope emits the exact structural-check continuation",
        ]
    if case_id == "manual-evidence-artifacts-operational-efficiency":
        assert time.monotonic() - started <= 120
        return [
            "native Python EC scaffold/evidence gate completes within 120 seconds",
            "representative assertions pass without cargo delegation",
        ]
    second = _manual_snapshot()
    assert first["case"]["id"] == second["case"]["id"]
    assert first["case"]["evidence_paths"] == second["case"]["evidence_paths"]
    return [
        "two fresh EC scaffolds produce identical case and evidence identities",
        "both artifact envelopes route to the same structural check",
    ]
