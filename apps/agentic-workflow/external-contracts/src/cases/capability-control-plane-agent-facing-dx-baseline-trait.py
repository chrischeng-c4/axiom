"""Black-box contract for the agent_facing profile trait's DX baseline derivation."""

from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path


CASE_ID = "capability-control-plane-agent-facing-dx-baseline-trait"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "agent-facing-dx-baseline-trait"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case capability-control-plane-agent-facing-dx-baseline-trait"
)
ASSERTIONS = (
    "a project without the agent_facing trait does not derive or require the developer-agent-experience baseline capability",
    "an explicit agent_facing profile trait derives the developer-agent-experience baseline capability",
    "a project missing the derived baseline capability reports the exact missing-baseline remediation blocker",
    "declaring the derived baseline capability in the capability document clears the remediation blocker",
)

MISSING_BASELINE_BLOCKER = (
    "capability profile requires missing baseline capabilities: "
    "developer-agent-experience"
)


def _repository_root() -> Path:
    return Path(__file__).resolve().parents[5]


def _aw_binary() -> Path:
    return _repository_root() / "target" / "debug" / "aw"


def _capability_document(*, with_baseline: bool) -> str:
    baseline_row = (
        "| Baseline | - | implemented | verified | smoke | ready | "
        "Derived developer-agent-experience baseline |\n"
        if with_baseline
        else ""
    )
    baseline_section = (
        """
### Baseline

ID: developer-agent-experience
Type: DeveloperTool
Surfaces: CLI: `aw capability report --project demo` - reports capability evidence.
EC Dimensions: behavior: `true` - baseline capability satisfying the agent_facing trait.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Serve as the required developer-agent-experience baseline for the agent_facing trait.
Gate Inventory:
- `true`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Baseline coverage | change | - | implemented | verified | smoke | `true` |
"""
        if with_baseline
        else ""
    )
    return f"""# Demo Capabilities

## Brief

Isolated agent_facing profile trait fixture.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Demo | - | implemented | verified | smoke | ready | Profile trait derivation fixture |
{baseline_row}
### Demo

ID: demo-capability
Type: DeveloperTool
Surfaces: CLI: `aw capability report --project demo` - reports capability evidence.
EC Dimensions: behavior: `true` - isolated black-box profile-trait contract.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Expose one capability unrelated to the derived baseline.
Gate Inventory:
- `true`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Demo coverage | change | - | implemented | verified | smoke | `true` |
{baseline_section}"""


def _write_fixture(
    root: Path, *, agent_facing: bool, with_baseline: bool
) -> None:
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
    project_config = (
        """[capability.profile]
traits = ["agent_facing"]
"""
        if agent_facing
        else "# no capability profile traits\n"
    )
    (project / "aw.toml").write_text(project_config, encoding="utf-8")
    (project / "CAPABILITIES.md").write_text(
        _capability_document(with_baseline=with_baseline), encoding="utf-8"
    )


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


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-python-dx-trait-") as raw_tmp:
        root = Path(raw_tmp)

        _write_fixture(root, agent_facing=False, with_baseline=False)
        no_trait = _report(root)
        assert MISSING_BASELINE_BLOCKER not in no_trait["blockers"], no_trait
        assert all(
            capability["id"] != "developer-agent-experience"
            for capability in no_trait["capabilities"]
        ), no_trait

        _write_fixture(root, agent_facing=True, with_baseline=False)
        missing = _report(root)
        assert missing["blockers"].count(MISSING_BASELINE_BLOCKER) == 1, missing
        assert all(
            capability["id"] != "developer-agent-experience"
            for capability in missing["capabilities"]
        ), missing

        _write_fixture(root, agent_facing=True, with_baseline=True)
        remediated = _report(root)
        assert MISSING_BASELINE_BLOCKER not in remediated["blockers"], remediated
        assert [
            capability["id"]
            for capability in remediated["capabilities"]
            if capability["id"] == "developer-agent-experience"
        ] == ["developer-agent-experience"], remediated

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
