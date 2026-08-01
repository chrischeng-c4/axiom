"""Black-box contract for cap_path default resolution and README migration (#1848)."""

from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path


CASE_ID = "capability-control-plane-default-cap-path-flips-to-capabilities-md"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "default-cap-path-flips-to-capabilities-md"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case capability-control-plane-default-cap-path-flips-to-capabilities-md"
)
ASSERTIONS = (
    "with no cap_path override, capability report resolves cap_path to <project path>/CAPABILITIES.md",
    "an explicit [[projects]].cap_path entry overrides the CAPABILITIES.md default",
    "a README-resident project with no CAPABILITIES.md degrades to a location-migration advisory finding, not a hard parse failure",
    "aw capability migrate relocates README-resident structure into CAPABILITIES.md and leaves a Capability Contract pointer in the README",
)


def _repository_root() -> Path:
    return Path(__file__).resolve().parents[5]


def _aw_binary() -> Path:
    return _repository_root() / "target" / "debug" / "aw"


def _capability_document() -> str:
    return """# Demo Capabilities

## Brief

Isolated cap_path resolution fixture.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Demo | - | implemented | verified | smoke | ready | cap_path resolution fixture |

### Demo

ID: demo-capability
Type: DeveloperTool
Surfaces: CLI: `aw capability report --project demo` - reports capability evidence.
EC Dimensions: behavior: `true` - isolated black-box cap_path contract.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Expose one capability used only to prove cap_path resolution.
Gate Inventory:
- `true`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Demo coverage | change | - | implemented | verified | smoke | `true` |
"""


def _root_aw_toml(*, cap_path: str | None) -> str:
    cap_path_line = f'cap_path = "{cap_path}"\n' if cap_path else ""
    return f"""version = "0.4.0"
interface = "cli"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "demo"
path = "project"
{cap_path_line}label = "app:demo"

[[projects.workspaces]]
name = "demo"
paths = ["project/**"]
target = "python"
test_cmd = "true"
"""


def _write_default_fixture(root: Path) -> None:
    project = root / "project"
    (root / ".git").mkdir(exist_ok=True)
    project.mkdir(parents=True, exist_ok=True)
    (root / "aw.toml").write_text(_root_aw_toml(cap_path=None), encoding="utf-8")
    (project / "CAPABILITIES.md").write_text(_capability_document(), encoding="utf-8")


def _write_override_fixture(root: Path) -> None:
    project = root / "project"
    (root / ".git").mkdir(exist_ok=True)
    project.mkdir(parents=True, exist_ok=True)
    (root / "aw.toml").write_text(
        _root_aw_toml(cap_path="project/CONTRACT.md"), encoding="utf-8"
    )
    (project / "CONTRACT.md").write_text(_capability_document(), encoding="utf-8")


def _write_readme_resident_fixture(root: Path) -> None:
    project = root / "project"
    (root / ".git").mkdir(exist_ok=True)
    project.mkdir(parents=True, exist_ok=True)
    (root / "aw.toml").write_text(_root_aw_toml(cap_path=None), encoding="utf-8")
    (project / "README.md").write_text(_capability_document(), encoding="utf-8")


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


def _migrate(root: Path) -> dict[str, object]:
    completed = subprocess.run(
        [str(_aw_binary()), "capability", "migrate", "--project", "demo"],
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    assert completed.returncode == 0, (completed.stdout, completed.stderr)
    return json.loads(completed.stdout)


def verify() -> list[str]:
    readme_resident_reason_fragment = (
        "capability structure is README-resident at"
    )

    with tempfile.TemporaryDirectory(prefix="aw-python-cap-path-default-") as raw_tmp:
        root = Path(raw_tmp)
        _write_default_fixture(root)
        report = _report(root)
        assert report["cap_path"].replace("\\", "/").endswith(
            "project/CAPABILITIES.md"
        ), report
        assert report["next_action"]["kind"] != "location_migration_required", report
        assert report["capability_count"] == 1, report
        assert report["capabilities"][0]["id"] == "demo-capability", report

    with tempfile.TemporaryDirectory(prefix="aw-python-cap-path-override-") as raw_tmp:
        root = Path(raw_tmp)
        _write_override_fixture(root)
        report = _report(root)
        assert report["cap_path"].replace("\\", "/").endswith(
            "project/CONTRACT.md"
        ), report
        assert report["next_action"]["kind"] != "location_migration_required", report
        assert report["capability_count"] == 1, report
        assert report["capabilities"][0]["id"] == "demo-capability", report

    with tempfile.TemporaryDirectory(prefix="aw-python-cap-path-readme-") as raw_tmp:
        root = Path(raw_tmp)
        _write_readme_resident_fixture(root)

        blocked = _report(root)
        assert blocked["status"] == "blocked", blocked
        assert blocked["next_action"]["kind"] == "location_migration_required", blocked
        assert (
            blocked["next_action"]["command"] == "aw capability migrate --project demo"
        ), blocked
        assert len(blocked["blockers"]) == 1, blocked
        assert readme_resident_reason_fragment in blocked["blockers"][0], blocked
        assert "CAPABILITIES.md" in blocked["blockers"][0], blocked

        migrated = _migrate(root)
        assert migrated["changed"] is True, migrated
        assert migrated["status"] == "migrated", migrated
        assert migrated["cap_path"].replace("\\", "/").endswith(
            "project/CAPABILITIES.md"
        ), migrated

        cap_body = (root / "project" / "CAPABILITIES.md").read_text(encoding="utf-8")
        assert "demo-capability" in cap_body, cap_body

        readme_body = (root / "project" / "README.md").read_text(encoding="utf-8")
        assert "## Capability Contract" in readme_body, readme_body
        assert "[CAPABILITIES.md](CAPABILITIES.md)" in readme_body, readme_body

        after_migrate = _report(root)
        joined_blockers = " ".join(after_migrate["blockers"])
        assert readme_resident_reason_fragment not in joined_blockers, after_migrate

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
