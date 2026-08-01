"""Black-box contract for the capability sweep human review queue grouping."""

from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path


CASE_ID = "capability-control-plane-capability-project-sweep"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "capability-project-sweep"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case capability-control-plane-capability-project-sweep"
)
ASSERTIONS = (
    "capability sweep reports one entry per configured project",
    "projects that land in the identical (status, next action) shape are grouped into one human sweep queue entry",
    "a project whose shape diverges splits into its own distinct sweep queue entry",
    "human mode renders the same sweep as a readable queue summary",
)


def _repository_root() -> Path:
    return Path(__file__).resolve().parents[5]


def _aw_binary() -> Path:
    return _repository_root() / "target" / "debug" / "aw"


def _capability_document() -> str:
    return """# Demo Capabilities

## Brief

Isolated capability sweep fixture.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Demo | - | implemented | verified | smoke | ready | Capability sweep fixture |

### Demo

ID: demo-capability
Type: DeveloperTool
Surfaces: CLI: `aw capability sweep --skip-issue-inventory` - reports capability sweep evidence.
EC Dimensions: behavior: `true` - isolated black-box sweep contract.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Expose one capability used only to prove sweep grouping.
Gate Inventory:
- `true`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Demo coverage | change | - | implemented | verified | smoke | `true` |
"""


def _write_fixture(root: Path, *, alpha_has_capability_doc: bool) -> None:
    alpha = root / "projects" / "alpha"
    beta = root / "projects" / "beta"
    (root / ".git").mkdir(exist_ok=True)
    alpha.mkdir(parents=True, exist_ok=True)
    beta.mkdir(parents=True, exist_ok=True)
    (root / "aw.toml").write_text(
        """version = "0.4.0"
interface = "cli"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "alpha"
path = "projects/alpha"
cap_path = "projects/alpha/CAPABILITIES.md"
label = "app:alpha"

[[projects.workspaces]]
name = "alpha"
paths = ["projects/alpha/**"]
target = "python"
test_cmd = "true"

[[projects]]
name = "beta"
path = "projects/beta"
cap_path = "projects/beta/CAPABILITIES.md"
label = "app:beta"

[[projects.workspaces]]
name = "beta"
paths = ["projects/beta/**"]
target = "python"
test_cmd = "true"
""",
        encoding="utf-8",
    )
    alpha_cap_path = alpha / "CAPABILITIES.md"
    if alpha_has_capability_doc:
        alpha_cap_path.write_text(_capability_document(), encoding="utf-8")
    elif alpha_cap_path.exists():
        alpha_cap_path.unlink()


def _sweep(root: Path, *, human: bool = False) -> subprocess.CompletedProcess[str]:
    args = [str(_aw_binary()), "capability", "sweep", "--skip-issue-inventory"]
    if human:
        args.append("--human")
    completed = subprocess.run(
        args,
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    assert completed.returncode in (0, 1), (completed.stdout, completed.stderr)
    return completed


def _sweep_json(root: Path) -> dict[str, object]:
    completed = _sweep(root, human=False)
    return json.loads(completed.stdout)


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-python-capability-sweep-") as raw_tmp:
        root = Path(raw_tmp)

        _write_fixture(root, alpha_has_capability_doc=False)
        same_shape = _sweep_json(root)
        assert same_shape["project_count"] == 2, same_shape
        assert len(same_shape["projects"]) == 2, same_shape
        assert len(same_shape["groups"]) == 1, same_shape
        only_group = same_shape["groups"][0]
        assert only_group["count"] == 2, only_group
        assert set(only_group["projects"]) == {"alpha", "beta"}, only_group

        _write_fixture(root, alpha_has_capability_doc=True)
        split_shape = _sweep_json(root)
        assert split_shape["project_count"] == 2, split_shape
        assert len(split_shape["groups"]) == 2, split_shape
        for group in split_shape["groups"]:
            assert group["count"] == 1, split_shape
        grouped_projects = sorted(
            group["projects"][0] for group in split_shape["groups"]
        )
        assert grouped_projects == ["alpha", "beta"], split_shape

        human = _sweep(root, human=True)
        assert human.stdout.startswith("capability sweep: "), human.stdout
        assert "projects complete]" in human.stdout.splitlines()[0], human.stdout
        assert "alpha" in human.stdout, human.stdout
        assert "beta" in human.stdout, human.stdout

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
