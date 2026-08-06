"""Black-box contract for the one-way WI<->capability reference direction (#1847)."""

from __future__ import annotations

import tempfile
from pathlib import Path

from wi_contract_fixture import (
    create,
    final_json,
    run_aw,
    write_python_artifact_lock,
    write_python_artifact_unit_test,
)


CASE_ID = "capability-control-plane-one-way-wi-reference-direction"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "one-way-wi-reference-direction"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case capability-control-plane-one-way-wi-reference-direction"
)
ASSERTIONS = (
    "an open gap with no doc-stored ref and no tracker WI routes to create_wi",
    "a WI that declares the capability/claim in its own body resolves as tracker-side wi_evidence with no doc-stored ref at all, and the capability document's WI cell is never mutated",
    "a stale/unresolvable doc-stored WI reference degrades to advisory unknown-state evidence instead of a hard failure",
)


def _capability_document(*, wi_cell: str) -> str:
    return f"""# Demo Capabilities

## Brief

Isolated WI-reference-direction fixture.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Demo | {wi_cell} | implemented | failing | smoke | blocked | WI-reference-direction fixture |

### Demo

ID: demo-capability
Type: DeveloperTool
Surfaces: CLI: `aw capability report --project demo --include-issue-inventory` - reports capability evidence.
EC Dimensions: behavior: `true` - isolated black-box WI-linkage contract.
Root WI: {wi_cell}
Status: auditing
Required Verification: smoke
Promise:
Expose one red gap used only to prove tracker-side WI evidence routing.
Gate Inventory:
- `true`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Fix red claim | change | {wi_cell} | implemented | failing | smoke | `true` |
"""


def _write_fixture(root: Path, *, wi_cell: str) -> None:
    project = root / "project"
    td_root = project / "tech-design"
    ec_root = project / "external-contracts"
    (root / ".git").mkdir(exist_ok=True)
    (td_root / "src/demo/public_contracts").mkdir(parents=True, exist_ok=True)
    (ec_root / "src/cases").mkdir(parents=True, exist_ok=True)
    (ec_root / "evidence").mkdir(exist_ok=True)
    (root / "aw.toml").write_text(
        """version = "0.4.0"
interface = "cli"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "demo"
path = "project"
td_path = "project/tech-design"
cap_path = "project/CAPABILITIES.md"
label = "app:demo"

[[projects.workspaces]]
name = "demo"
paths = ["project/**"]
target = "python"
test_cmd = "true"
""",
        encoding="utf-8",
    )
    (project / "CAPABILITIES.md").write_text(
        _capability_document(wi_cell=wi_cell), encoding="utf-8"
    )
    # A minimal, valid, already-evidenced Python TD/EC skeleton keeps the
    # shared readiness projection (`python-artifact-readiness`) from
    # dominating `next_action` with an `aw ec check`/`aw ec verify`
    # remediation before the WI-evidence gap loop this case actually
    # targets is reached: a zero-case inventory is itself a finding
    # ("declares no cases"), so at least one case with real evidence is
    # required for `python_artifact.ready` to become `true`.
    (td_root / "pyproject.toml").write_text(
        """[project]
name = "demo-tech-design"
version = "0.1.0"
requires-python = ">=3.11"
""",
        encoding="utf-8",
    )
    (td_root / "src/demo/public_contracts/claim.py").write_text(
        '''__aw_artifact_id__ = "artifact:demo/claim"
__aw_public_contract__ = True


def demo_claim() -> str:
    return "Demo claim"
''',
        encoding="utf-8",
    )
    (ec_root / "src/runner.py").write_text(
        'print("fixture runner")\n',
        encoding="utf-8",
    )
    (ec_root / "src/cases/claim.py").write_text(
        'def verify() -> list[str]:\n    return ["claim is externally observable"]\n',
        encoding="utf-8",
    )
    (ec_root / "evidence/claim.json").write_text(
        '{"protocol":"aw.python-ec.evidence.v1","exit_code":0}\n',
        encoding="utf-8",
    )
    (ec_root / "pyproject.toml").write_text(
        """[project]
name = "demo-external-contracts"
version = "0.1.0"
requires-python = ">=3.11"

[tool.aw.python-artifact]
protocol = "aw.python-artifact.v1"
entrypoint = "src/runner.py"
source_roots = ["src"]
dependency_files = ["pyproject.toml", "uv.lock"]
evidence_dir = "evidence"

[tool.aw.python-ec]
protocol = "aw.python-ec.v1"
author = "fixture:external"
efficiency_policy = "not-applicable"

[[tool.aw.python-ec.cases]]
id = "demo-coverage"
artifact_id = "artifact:demo/claim"
capability_id = "demo-capability"
use_case_id = "fix-red-claim"
dimension = "behavior"
applicability = "td"
test_path = "src/cases/claim.py"
promise = "the WI-reference-direction fixture has passing coverage"
oracle = "the outer EC independently inspects the real aw process output"
target = "python"
command = "true"
evidence_paths = ["evidence/claim.json"]
""",
        encoding="utf-8",
    )
    write_python_artifact_lock(ec_root, name="demo-external-contracts")
    write_python_artifact_unit_test(ec_root, "claim")


def _report(root: Path) -> dict[str, object]:
    completed = run_aw(
        root,
        "capability",
        "report",
        "--project",
        "demo",
        "--include-issue-inventory",
        expect_success=None,
    )
    assert completed.returncode == 0, (completed.stdout, completed.stderr)
    return final_json(completed)


def _demo(report: dict[str, object]) -> dict[str, object]:
    return next(c for c in report["capabilities"] if c["id"] == "demo-capability")


def verify() -> list[str]:
    # Part A: an open gap with neither a doc-stored WI ref nor any tracker WI
    # at all has nothing to continue, so it routes to WI creation.
    with tempfile.TemporaryDirectory(prefix="aw-python-wi-ref-a-") as raw_tmp:
        root = Path(raw_tmp)
        _write_fixture(root, wi_cell="-")
        report = _report(root)
        demo = _demo(report)
        assert demo.get("wi_evidence", []) == [], demo
        assert report["next_action"]["kind"] == "create_wi", report
        assert report["next_action"]["command"] == "aw wi plan --project demo", report

    # Part B: a WI declares the capability/claim in its own body via
    # `## Capability Alignment`. The doc row's WI cell is never touched, yet
    # the WI resolves as tracker-side evidence and the gap stops routing to
    # `create_wi` -- tracker-side provenance wins over an absent doc ref.
    with tempfile.TemporaryDirectory(prefix="aw-python-wi-ref-b-") as raw_tmp:
        root = Path(raw_tmp)
        _write_fixture(root, wi_cell="-")
        before = (root / "project" / "CAPABILITIES.md").read_text(encoding="utf-8")

        body = (
            "## Problem\n\nThe red claim needs a fix.\n\n"
            "## Capability Alignment\n\n"
            "Capability: `demo-capability`\n"
            "Capability Gap: `fix-red-claim`\n"
            "Progress Evidence: independently observed via wi_evidence resolution\n\n"
            "## Acceptance Criteria\n\n"
            "- AC1: the WI body is the tracker-side source of capability alignment evidence.\n"
        )
        wi = create(root, "Fix red claim", "change", "--body", body)
        slug = wi["slug"]
        # Local-only WIs are created as drafts; open it so it is eligible as
        # the preferred tracker-side match ahead of any (absent) doc ref.
        # `wi update` prints a plain confirmation line, not JSON, so only its
        # exit code is asserted here (via `run_aw`'s default expect_success).
        run_aw(root, "wi", "update", slug, "--state", "open")

        report = _report(root)
        demo = _demo(report)
        entry = next(e for e in demo["wi_evidence"] if e["gap_id"] == "fix-red-claim")
        assert entry["state"] == "open", entry
        assert entry["issue_type"] == "change", entry
        assert entry["title"] == "Fix red claim", entry
        assert entry["reference"] in demo["wi_refs"], demo
        assert report["next_action"]["kind"] != "create_wi", report

        after = (root / "project" / "CAPABILITIES.md").read_text(encoding="utf-8")
        assert after == before, after

    # Part C: a stale/unresolvable doc-stored WI reference (no matching
    # tracker WI at all) degrades to advisory unknown-state evidence; the
    # command still completes rather than hard-failing on the malformed cell.
    with tempfile.TemporaryDirectory(prefix="aw-python-wi-ref-c-") as raw_tmp:
        root = Path(raw_tmp)
        _write_fixture(root, wi_cell="#99999")
        report = _report(root)
        demo = _demo(report)
        entry = next(
            e
            for e in demo["wi_evidence"]
            if e["gap_id"] == "fix-red-claim" and e["reference"] == "#99999"
        )
        assert entry["issue_type"] == "unknown", entry
        assert entry["state"] == "unknown", entry
        assert entry["title"] == "", entry
        assert report["status"] == "blocked", report
        assert report["blockers"] == [], report
        assert report["next_action"]["kind"] == "reconcile_wi_refs", report
        assert report["next_action"]["capability_id"] == "demo-capability", report
        assert report["next_action"]["gap_id"] == "fix-red-claim", report
        assert report["next_action"]["command"] == "aw wi plan --project demo", report
        assert report["next_action"]["requires_hitl"] is False, report
        assert report["next_action"]["reason"] == (
            "active WI reference is not present in project issue inventory; "
            "sync or recreate a bounded WI before the EC-first TD/codegen "
            "lifecycle; once the correct WI id is known, resolve without a raw "
            "file edit via `aw capability set-wi-ref --project demo --capability "
            "demo-capability --claim fix-red-claim --wi <id>`"
        ), report

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
