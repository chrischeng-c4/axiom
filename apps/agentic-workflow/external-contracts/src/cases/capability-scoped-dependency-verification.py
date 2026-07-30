"""Black-box contract for dependency-closed capability verification."""

from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path


CASE_ID = "capability-scoped-dependency-verification"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "scoped-capability-verification"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "python3 apps/agentic-workflow/external-contracts/src/runner.py "
    "--case capability-scoped-dependency-verification"
)
ASSERTIONS = (
    "requested capability and transitive dependency gates execute",
    "unrelated capability gate does not execute",
    "workspace project test gates are excluded",
    "scoped report contains only the dependency closure",
    "unknown capability fails before any gate executes",
    "full-project verification still executes workspace test gates",
    "goal progress, blocked next, and typed transition retain --capability root",
)


def _repository_root() -> Path:
    return Path(__file__).resolve().parents[5]


def _aw_binary() -> Path:
    return _repository_root() / "target" / "debug" / "aw"


def _capability_document(
    leaf_marker: Path,
    middle_marker: Path,
    root_marker: Path,
    unrelated_marker: Path,
) -> str:
    return f"""# Scoped Verification Fixture

## Capability: Leaf
<!-- type: capability lang: yaml -->

```yaml
id: leaf
status: verified
capability_type: AgentFirst
surfaces:
  - kind: CLI
    commands: ["demo leaf"]
    summary: "Leaf fixture command."
ec_dimensions:
  - dimension: behavior
    runner: "true"
    summary: "Leaf fixture behavior."
promise: "Leaf verifies."
current_state: "Ready."
gaps:
  - id: leaf-root
    status: closed
    summary: "Leaf root"
evidence:
  verification:
    - id: leaf-gate
      command: "touch {leaf_marker}"
      proves: "leaf ran"
```

## Capability: Middle
<!-- type: capability lang: yaml -->

```yaml
id: middle
status: verified
capability_type: AgentFirst
surfaces:
  - kind: CLI
    commands: ["demo middle"]
    summary: "Middle fixture command."
ec_dimensions:
  - dimension: behavior
    runner: "true"
    summary: "Middle fixture behavior."
promise: "Middle verifies."
current_state: "Ready."
dependencies: [leaf]
gaps:
  - id: middle-root
    status: closed
    summary: "Middle root"
evidence:
  verification:
    - id: middle-gate
      command: "touch {middle_marker}"
      proves: "middle ran"
```

## Capability: Root
<!-- type: capability lang: yaml -->

```yaml
id: root
status: verified
capability_type: AgentFirst
surfaces:
  - kind: CLI
    commands: ["demo root"]
    summary: "Root fixture command."
ec_dimensions:
  - dimension: behavior
    runner: "true"
    summary: "Root fixture behavior."
promise: "Root verifies."
current_state: "Ready."
dependencies: [middle]
gaps:
  - id: root-root
    status: closed
    summary: "Root root"
evidence:
  verification:
    - id: root-gate
      command: "touch {root_marker}"
      proves: "root ran"
```

## Capability: Unrelated
<!-- type: capability lang: yaml -->

```yaml
id: unrelated
status: verified
capability_type: AgentFirst
surfaces:
  - kind: CLI
    commands: ["demo unrelated"]
    summary: "Unrelated fixture command."
ec_dimensions:
  - dimension: behavior
    runner: "true"
    summary: "Unrelated fixture behavior."
promise: "Unrelated must not run."
current_state: "Ready."
gaps:
  - id: unrelated-root
    status: closed
    summary: "Unrelated root"
evidence:
  verification:
    - id: unrelated-gate
      command: "touch {unrelated_marker}"
      proves: "unrelated ran"
```
"""


def _run(
    fixture_root: Path,
    cap_path: Path,
    capability_id: str | None,
) -> subprocess.CompletedProcess[str]:
    command = [
        str(_aw_binary()),
        "capability",
        "check",
        "--project",
        "demo",
        "--cap-path",
        str(cap_path),
        "--verify",
        "--skip-issue-inventory",
    ]
    if capability_id is not None:
        command.extend(["--capability", capability_id])
    return subprocess.run(
        command,
        cwd=fixture_root,
        capture_output=True,
        text=True,
        check=False,
    )


def _migrate(fixture_root: Path, cap_path: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            str(_aw_binary()),
            "capability",
            "migrate",
            "--project",
            "demo",
            "--cap-path",
            str(cap_path),
        ],
        cwd=fixture_root,
        capture_output=True,
        text=True,
        check=False,
    )


def _goal(
    fixture_root: Path,
    capability_id: str,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            str(_aw_binary()),
            "goal",
            "capability",
            capability_id,
            "--project",
            "demo",
        ],
        cwd=fixture_root,
        capture_output=True,
        text=True,
        check=False,
    )


def _write_td_links(fixture_root: Path) -> None:
    td_root = fixture_root / "tech-design"
    for capability_id in ("leaf", "middle", "root", "unrelated"):
        claim_id = f"{capability_id}-root"
        (td_root / f"{capability_id}.md").write_text(
            f"""---
id: {capability_id}-td
capability_refs:
  - id: {capability_id}
    role: primary
    gap: {claim_id}
    claim: {claim_id}
    coverage: full
    rationale: "Fixture TD links the executable capability claim."
---

# {capability_id.title()} fixture TD
""",
            encoding="utf-8",
        )


def _write_python_artifacts(fixture_root: Path) -> None:
    td_source = fixture_root / "tech-design" / "src"
    td_source.mkdir()
    (td_source / "design.py").write_text(
        '__aw_artifact_id__ = "artifact:demo/scoped-capability-fixture"\n',
        encoding="utf-8",
    )

    ec_root = fixture_root / "external-contracts"
    case_root = ec_root / "src" / "cases"
    evidence_root = ec_root / "evidence"
    case_root.mkdir(parents=True)
    evidence_root.mkdir()
    (ec_root / "src" / "runner.py").write_text(
        'print("fixture runner")\n',
        encoding="utf-8",
    )
    (case_root / "demo.py").write_text(
        'def verify() -> list[str]:\n    return ["fixture"]\n',
        encoding="utf-8",
    )
    (evidence_root / "demo.json").write_text('{"status":"passed"}\n', encoding="utf-8")
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
id = "demo-behavior"
artifact_id = "artifact:demo/scoped-capability-fixture"
capability_id = "root"
use_case_id = "scoped-verification"
dimension = "behavior"
applicability = "td"
test_path = "src/cases/demo.py"
promise = "the fixture has one structurally valid behavior case"
oracle = "the outer EC independently checks the real aw process"
target = "python"
command = "true"
evidence_paths = ["evidence/demo.json"]
""",
        encoding="utf-8",
    )


def _blocked_goal_document() -> str:
    return """# Scoped Goal Blocker

## Brief

Fixture used to inspect the scoped goal envelope.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Root | - | implemented | verified | smoke | ready | verified |

### Root

ID: root
Type: AgentFirst
Root WI: -
Status: verified
Required Verification: smoke
Surfaces:
- CLI: `demo root` - scoped fixture command.
EC Dimensions:
- behavior: `true` - scoped fixture behavior.
Promise:
Root remains a valid capability while a relevant TD reference is malformed.
Gate Inventory:
- CAPABILITIES.md
"""


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-scoped-capability-") as raw_tmp:
        tmp = Path(raw_tmp)
        leaf_marker = tmp / "leaf-ran"
        middle_marker = tmp / "middle-ran"
        root_marker = tmp / "root-ran"
        unrelated_marker = tmp / "unrelated-ran"
        workspace_marker = tmp / "workspace-ran"
        cap_path = tmp / "CAPABILITIES.md"
        (tmp / ".git").mkdir()
        (tmp / "tech-design").mkdir()
        (tmp / "aw.toml").write_text(
            f"""version = "0.3.60"
interface = "cli"

[agentic_workflow.issue_platform]
type = "local"

[[projects]]
name = "demo"
path = "."
td_path = "tech-design"
cap_path = "CAPABILITIES.md"
label = "app:demo"

[[projects.workspaces]]
name = "demo"
paths = ["**"]
target = "python"
test_cmd = "touch {workspace_marker}"
""",
            encoding="utf-8",
        )
        cap_path.write_text(
            _capability_document(
                leaf_marker,
                middle_marker,
                root_marker,
                unrelated_marker,
            ),
            encoding="utf-8",
        )
        migrated = _migrate(tmp, cap_path)
        assert migrated.returncode == 0, (migrated.stdout, migrated.stderr)
        _write_td_links(tmp)
        _write_python_artifacts(tmp)

        completed = _run(tmp, cap_path, "root")
        assert completed.returncode == 0, (completed.stdout, completed.stderr)
        output_lines = [
            line for line in completed.stdout.splitlines() if line.startswith("{")
        ]
        if not output_lines:
            raise AssertionError(
                f"scoped capability check emitted no JSON: {completed.stderr}"
            )
        report = json.loads(output_lines[-1])
        assert report["status"] == "healthy", report
        assert leaf_marker.is_file()
        assert middle_marker.is_file()
        assert root_marker.is_file()
        assert not unrelated_marker.exists()
        assert not workspace_marker.exists()
        assert report["test_gates"]["command_count"] == 0
        assert report["test_gates"]["status"] == "passed"
        assert {
            capability["id"] for capability in report["capabilities"]
        } == {"leaf", "middle", "root"}

        leaf_marker.unlink()
        middle_marker.unlink()
        root_marker.unlink()
        unknown = _run(tmp, cap_path, "unknown")
        assert unknown.returncode != 0
        assert "capability `unknown` is not declared" in unknown.stderr
        assert not leaf_marker.exists()
        assert not middle_marker.exists()
        assert not root_marker.exists()
        assert not unrelated_marker.exists()
        assert not workspace_marker.exists()

        full_project = _run(tmp, cap_path, None)
        assert full_project.returncode == 0, (
            full_project.stdout,
            full_project.stderr,
        )
        full_output_lines = [
            line for line in full_project.stdout.splitlines() if line.startswith("{")
        ]
        assert full_output_lines
        full_report = json.loads(full_output_lines[-1])
        assert full_report["status"] == "healthy", full_report
        assert workspace_marker.is_file(), (
            full_project.stdout,
            full_project.stderr,
        )

        cap_path.write_text(_blocked_goal_document(), encoding="utf-8")
        bad_td = tmp / "tech-design" / "root-blocker.md"
        bad_td.write_text(
            """---
id: root-blocker
capability_refs:
  - id: root
    role: primary
    gap: not-declared
---

# Scoped blocker
""",
            encoding="utf-8",
        )
        goal = _goal(tmp, "root")
        assert goal.returncode == 0, (goal.stdout, goal.stderr)
        events = [
            json.loads(line)
            for line in goal.stdout.splitlines()
            if line.startswith("{")
        ]
        progress = [
            event
            for event in events
            if event.get("event") == "progress"
            and event.get("phase") == "capability"
        ]
        assert progress
        expected_command = (
            "aw capability check --project demo --verify --capability root"
        )
        assert all(event.get("command") == expected_command for event in progress)
        envelope = events[-1]
        assert envelope["status"] == "blocked", envelope
        assert envelope["next"]["command"] == expected_command
        assert (
            envelope["prompt_contract"]["transition"]["command"] == expected_command
        )

    return list(ASSERTIONS)
