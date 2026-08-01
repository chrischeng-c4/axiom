"""Black-box contract for the shared Python TD/EC readiness projection (#2304)."""

from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path

from wi_contract_fixture import (
    write_python_artifact_lock,
    write_python_artifact_unit_test,
)


CASE_ID = "capability-control-plane-python-artifact-readiness"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "python-artifact-readiness"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case capability-control-plane-python-artifact-readiness"
)
ASSERTIONS = (
    "capability report and health spec expose the identical python_artifact/python_spec readiness projection",
    "a project with no Python TD/EC inventory reports the exact missing-inventory blockers with an aw ec check remediation",
    "a required EC case with missing evidence reports the exact missing-evidence blocker with an aw ec verify --stage td remediation",
    "writing real evidence for the missing case clears the blocker and flips readiness to ready in both surfaces",
)


def _repository_root() -> Path:
    return Path(__file__).resolve().parents[5]


def _aw_binary() -> Path:
    return _repository_root() / "target" / "debug" / "aw"


def _capability_document() -> str:
    return """# Demo Capabilities

## Brief

Isolated Python artifact readiness fixture.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Demo | - | implemented | verified | smoke | ready | Python artifact readiness fixture |

### Demo

ID: demo-capability
Type: DeveloperTool
Surfaces: CLI: `aw capability report --project demo` - reports capability evidence.
EC Dimensions: behavior: `true` - isolated black-box readiness contract.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Expose one Python-authored behavior guarded by readiness evidence.
Gate Inventory:
- `true`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Readiness coverage | change | - | implemented | verified | smoke | `true` |
"""


def _write_no_inventory_fixture(root: Path) -> None:
    project = root / "project"
    (root / ".git").mkdir(exist_ok=True)
    project.mkdir(parents=True, exist_ok=True)
    (root / "aw.toml").write_text(
        """version = "0.4.0"
interface = "cli"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "demo"
path = "project"
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


def _write_skeleton_fixture(root: Path, *, with_evidence: bool) -> None:
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
    (td_root / "src/demo/public_contracts/readiness.py").write_text(
        '''__aw_artifact_id__ = "artifact:demo/readiness"
__aw_public_contract__ = True


def demo_readiness() -> str:
    return "Python artifact readiness"
''',
        encoding="utf-8",
    )
    (ec_root / "src/runner.py").write_text(
        'print("fixture runner")\n',
        encoding="utf-8",
    )
    (ec_root / "src/cases/readiness.py").write_text(
        'def verify() -> list[str]:\n    return ["readiness is externally observable"]\n',
        encoding="utf-8",
    )
    if with_evidence:
        (ec_root / "evidence/readiness.json").write_text(
            '{"protocol":"aw.python-ec.evidence.v1","exit_code":0}\n',
            encoding="utf-8",
        )
    else:
        stale_evidence = ec_root / "evidence/readiness.json"
        if stale_evidence.exists():
            stale_evidence.unlink()
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
id = "demo-readiness"
artifact_id = "artifact:demo/readiness"
capability_id = "demo-capability"
use_case_id = "demo-readiness"
dimension = "behavior"
applicability = "td"
test_path = "src/cases/readiness.py"
promise = "the readiness projection reports this case's evidence state"
oracle = "the outer EC independently inspects the real aw process output"
target = "python"
command = "true"
evidence_paths = ["evidence/readiness.json"]
""",
        encoding="utf-8",
    )
    write_python_artifact_lock(ec_root, name="demo-external-contracts")
    write_python_artifact_unit_test(ec_root, "readiness")


def _report(root: Path) -> dict[str, object]:
    completed = subprocess.run(
        [
            str(_aw_binary()),
            "capability",
            "report",
            "--project",
            "demo",
            "--skip-issue-inventory",
        ],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    assert completed.returncode == 0, (completed.stdout, completed.stderr)
    reports = [
        json.loads(line)
        for line in completed.stdout.splitlines()
        if line.startswith("{")
    ]
    assert reports, completed.stdout
    return reports[-1]


def _health_spec(root: Path) -> dict[str, object]:
    completed = subprocess.run(
        [str(_aw_binary()), "health", "--project", "demo", "spec"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    assert completed.returncode in (0, 1), (completed.stdout, completed.stderr)
    return json.loads(completed.stdout)


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-python-artifact-readiness-") as raw_tmp:
        root = Path(raw_tmp)

        _write_no_inventory_fixture(root)
        report = _report(root)
        health = _health_spec(root)
        python_artifact = report["python_artifact"]
        assert python_artifact == health["data"]["python_spec"], (
            python_artifact,
            health,
        )
        assert python_artifact["enabled"] is True, python_artifact
        assert python_artifact["ready"] is False, python_artifact
        assert any(
            blocker.startswith("Python EC inventory unavailable:")
            for blocker in python_artifact["blockers"]
        ), python_artifact
        assert any(
            blocker.startswith("Python TD inventory unavailable:")
            for blocker in python_artifact["blockers"]
        ), python_artifact
        assert python_artifact["next_command"] == "aw ec check --project demo", (
            python_artifact
        )

        _write_skeleton_fixture(root, with_evidence=False)
        missing_evidence_report = _report(root)
        missing_evidence = missing_evidence_report["python_artifact"]
        missing_evidence_health = _health_spec(root)
        assert missing_evidence == missing_evidence_health["data"]["python_spec"], (
            missing_evidence,
            missing_evidence_health,
        )
        assert missing_evidence["enabled"] is True, missing_evidence
        assert missing_evidence["ready"] is False, missing_evidence
        assert (
            "Python EC case `demo-readiness` has missing or empty digest-bound evidence"
            in missing_evidence["blockers"]
        ), missing_evidence
        assert missing_evidence["next_command"] == (
            "aw ec verify --project demo --stage td"
        ), missing_evidence

        _write_skeleton_fixture(root, with_evidence=True)
        ready_report = _report(root)
        ready = ready_report["python_artifact"]
        ready_health = _health_spec(root)
        assert ready == ready_health["data"]["python_spec"], (ready, ready_health)
        assert ready["enabled"] is True, ready
        assert ready["ready"] is True, ready
        assert ready["blockers"] == [], ready
        assert ready["next_command"] is None, ready

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
