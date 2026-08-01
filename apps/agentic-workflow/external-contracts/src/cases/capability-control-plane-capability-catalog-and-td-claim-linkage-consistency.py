"""Black-box contract for claim-closure validators and the claim reconciliation producer."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
from pathlib import Path

from wi_contract_fixture import (
    write_python_artifact_lock,
    write_python_artifact_unit_test,
)


CASE_ID = "capability-control-plane-capability-catalog-and-td-claim-linkage-consistency"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "capability-catalog-and-td-claim-linkage-consistency"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case capability-control-plane-capability-catalog-and-td-claim-linkage-consistency"
)
ASSERTIONS = (
    "a production EC case with no capability_id is rejected as unmapped",
    "a production EC case referencing an unknown capability is rejected by id",
    "a production EC case referencing an unknown claim under a known capability is rejected by id",
    "a production EC case that correctly names a real capability and claim becomes that claim's ec_case_ids evidence",
    "the read-only claim reconciliation producer independently reports supplemental drift evidence over the real inventory rather than acting as claim closure's sole oracle",
)


def _repository_root() -> Path:
    return Path(__file__).resolve().parents[5]


def _aw_binary() -> Path:
    return _repository_root() / "target" / "debug" / "aw"


def _capability_document() -> str:
    return """# Demo Capabilities

## Brief

Isolated claim-closure linkage fixture.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Demo | - | implemented | verified | smoke | ready | Claim-closure linkage fixture |

### Demo

ID: demo-capability
Type: DeveloperTool
Surfaces: CLI: `aw health --project demo claims` - reports claim closure evidence.
EC Dimensions: behavior: `true` - isolated black-box claim-linkage contract.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Expose one claim used only to prove claim-closure linkage validation.
Gate Inventory:
- `true`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Demo coverage | change | - | implemented | verified | smoke | `true` |
"""


def _write_fixture(root: Path) -> None:
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
    (project / "CAPABILITIES.md").write_text(_capability_document(), encoding="utf-8")
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
id = "demo-mapped-case"
artifact_id = "artifact:demo/claim"
capability_id = "demo-capability"
use_case_id = "demo-coverage"
dimension = "behavior"
applicability = "td"
test_path = "src/cases/claim.py"
promise = "the correctly mapped case becomes claim evidence"
oracle = "the outer EC independently inspects the real aw process output"
target = "python"
command = "true"
evidence_paths = ["evidence/claim.json"]

[[tool.aw.python-ec.cases]]
id = "demo-unmapped-case"
artifact_id = "artifact:demo/claim"
capability_id = "unmapped"
use_case_id = "n-a"
dimension = "behavior"
applicability = "td"
test_path = "src/cases/claim.py"
promise = "an unmapped case must be rejected by claim closure"
oracle = "the outer EC independently inspects the real aw process output"
target = "python"
command = "true"
evidence_paths = ["evidence/claim.json"]

[[tool.aw.python-ec.cases]]
id = "demo-unknown-capability-case"
artifact_id = "artifact:demo/claim"
capability_id = "nonexistent-capability"
use_case_id = "n-a"
dimension = "behavior"
applicability = "td"
test_path = "src/cases/claim.py"
promise = "a case naming an unknown capability must be rejected by claim closure"
oracle = "the outer EC independently inspects the real aw process output"
target = "python"
command = "true"
evidence_paths = ["evidence/claim.json"]

[[tool.aw.python-ec.cases]]
id = "demo-unknown-claim-case"
artifact_id = "artifact:demo/claim"
capability_id = "demo-capability"
use_case_id = "nonexistent-claim"
dimension = "behavior"
applicability = "td"
test_path = "src/cases/claim.py"
promise = "a case naming an unknown claim under a known capability must be rejected by claim closure"
oracle = "the outer EC independently inspects the real aw process output"
target = "python"
command = "true"
evidence_paths = ["evidence/claim.json"]
""",
        encoding="utf-8",
    )
    write_python_artifact_lock(ec_root, name="demo-external-contracts")
    write_python_artifact_unit_test(ec_root, "claim")


def _health_claims(root: Path) -> dict[str, object]:
    completed = subprocess.run(
        [str(_aw_binary()), "health", "--project", "demo", "claims"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    assert completed.returncode in (0, 1), (completed.stdout, completed.stderr)
    return json.loads(completed.stdout)


def _claim_reconciliation_report() -> dict[str, object]:
    script = (
        _repository_root()
        / "apps/agentic-workflow/external-contracts/src/claim_reconciliation.py"
    )
    completed = subprocess.run(
        [sys.executable, str(script)],
        cwd=_repository_root(),
        capture_output=True,
        text=True,
        check=False,
    )
    assert completed.returncode in (0, 1), (completed.stdout, completed.stderr)
    return json.loads(completed.stdout)


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-python-claim-catalog-") as raw_tmp:
        root = Path(raw_tmp)
        _write_fixture(root)
        payload = _health_claims(root)
        data = payload["data"]
        blockers = data["blockers"]
        claims = data["claims"]

        assert (
            "claim closure EC case `demo-unmapped-case` is unmapped; "
            "production cases must name capability_id and claim_id" in blockers
        ), blockers
        assert (
            "claim closure EC case `demo-unknown-capability-case` references "
            "unknown capability `nonexistent-capability`" in blockers
        ), blockers
        assert (
            "claim closure EC case `demo-unknown-claim-case` references unknown "
            "claim `nonexistent-claim` for capability `demo-capability`" in blockers
        ), blockers
        assert not any(
            "demo-mapped-case" in blocker
            and ("is unmapped" in blocker or "unknown" in blocker)
            for blocker in blockers
        ), blockers

        entry = next(
            claim
            for claim in claims
            if claim["capability_id"] == "demo-capability"
            and claim["claim_id"] == "demo-coverage"
        )
        assert entry["ec_case_ids"] == ["demo-mapped-case"], entry

    reconciliation = _claim_reconciliation_report()
    assert reconciliation["schema_version"] == (
        "aw.python-ec.claim-reconciliation.v2"
    ), reconciliation
    assert reconciliation["status"] == "clean", reconciliation
    assert reconciliation["case_count"] >= 66, reconciliation
    assert reconciliation["next"] is None, reconciliation

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
