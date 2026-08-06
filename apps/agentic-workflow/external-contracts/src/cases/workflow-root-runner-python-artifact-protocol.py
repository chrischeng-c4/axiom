"""Black-box contract for the Python artifact project protocol (#3298)."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import (
    final_json,
    project_fixture,
    run_aw,
    write_python_artifact_lock,
    write_python_artifact_unit_test,
)


CASE_ID = "workflow-root-runner-python-artifact-protocol"
CAPABILITY_ID = "workflow-root-runner"
USE_CASE_ID = "python-artifact-protocol"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case workflow-root-runner-python-artifact-protocol"
)
ASSERTIONS = (
    "a project-local external-contracts pyproject.toml that declares [tool.aw.python-artifact] with a safe project-relative entrypoint, source_roots, dependency_files (including pyproject.toml and uv.lock), and evidence_dir is independently discovered and structurally accepted by the real aw ec check surface",
    "mutating only the declared entrypoint to a path that escapes the project root (../escape.py) makes that same real aw ec check surface fail closed with the exact protocol error naming entrypoint as an unsafe project-relative path, before any EC-specific case content is even considered",
    "independently, mutating only dependency_files to omit the required uv.lock entry makes the same real aw ec check surface fail closed with the exact protocol error naming the missing uv.lock dependency file, and restoring both declarations verbatim makes the identical project pass the identical check again",
)

_ARTIFACT_TABLE = """[tool.aw.python-artifact]
protocol = "aw.python-artifact.v1"
entrypoint = "src/runner.py"
source_roots = ["src"]
dependency_files = ["pyproject.toml", "uv.lock"]
evidence_dir = "evidence"
"""

_CAPABILITIES_DOCUMENT = """# Artifact Protocol Fixture Capabilities

## Brief

Isolated Python-artifact-protocol fixture.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Artifact Protocol Fixture | - | planned | none | smoke | blocked | artifact-protocol fixture |

### Artifact Protocol Fixture

ID: demo-fixture-capability
Type: DeveloperTool
Surfaces: CLI: `aw ec check` - discovers and structurally validates the Python artifact/EC inventory.
EC Dimensions: behavior: `true` - isolated black-box artifact-protocol contract.
Root WI: -
Status: planned
Required Verification: smoke
Promise:
Prove safe source/dependency/entrypoint/evidence boundary declarations are required.
Gate Inventory:
- `true`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Demo fixture case | change | - | planned | none | smoke | `true` |
"""

_PYPROJECT = (
    "[project]\n"
    'name = "demo-external-contracts"\n'
    'version = "0.1.0"\n'
    'requires-python = ">=3.11"\n'
    "\n"
    f"{_ARTIFACT_TABLE}\n"
    "[tool.aw.python-ec]\n"
    'protocol = "aw.python-ec.v1"\n'
    'author = "fixture:python-artifact-protocol"\n'
    'efficiency_policy = "optional"\n'
    "\n"
    "[[tool.aw.python-ec.cases]]\n"
    'id = "demo-fixture-case"\n'
    'artifact_id = "artifact:demo/demo-external-contracts"\n'
    'capability_id = "demo-fixture-capability"\n'
    'use_case_id = "demo-fixture-case"\n'
    'dimension = "behavior"\n'
    'applicability = "td"\n'
    'test_path = "src/demo_fixture_case.py"\n'
    'promise = "the fixture EC case exists so the artifact-protocol assertions are reachable"\n'
    'oracle = "the outer EC independently inspects the real aw ec check output"\n'
    'target = "rust"\n'
    'command = "test -s external-contracts/evidence/demo-fixture-case.json"\n'
    'evidence_paths = ["evidence/demo-fixture-case.json"]\n'
)


def _write_project(ec_root: Path) -> Path:
    ec_root.mkdir(parents=True, exist_ok=True)
    pyproject_path = ec_root / "pyproject.toml"
    pyproject_path.write_text(_PYPROJECT, encoding="utf-8")
    (ec_root / "src").mkdir(parents=True, exist_ok=True)
    (ec_root / "src" / "runner.py").write_text("# fixture entrypoint\n", encoding="utf-8")
    (ec_root / "src" / "demo_fixture_case.py").write_text(
        'def verify() -> list[str]:\n    return ["fixture case executed"]\n',
        encoding="utf-8",
    )
    (ec_root / "evidence").mkdir(parents=True, exist_ok=True)
    write_python_artifact_lock(ec_root, name="demo-external-contracts")
    write_python_artifact_unit_test(ec_root)
    return pyproject_path


def verify() -> list[str]:
    with project_fixture() as root:
        (root / "CAPABILITIES.md").write_text(_CAPABILITIES_DOCUMENT, encoding="utf-8")
        pyproject_path = _write_project(root / "external-contracts")

        # Positive control: a project declaring safe source/dependency/
        # entrypoint/evidence boundaries is independently discovered and
        # structurally accepted by the real `aw ec check` surface.
        clean_check = final_json(run_aw(root, "ec", "check", "--project", "demo", "--json"))
        assert clean_check["clean"] is True, clean_check

        original = pyproject_path.read_text(encoding="utf-8")

        # Negative 1: an entrypoint that escapes the project root fails
        # closed with the exact protocol error, before any EC-specific case
        # content is even considered.
        escaped_entrypoint = original.replace(
            'entrypoint = "src/runner.py"', 'entrypoint = "../escape.py"', 1
        )
        assert escaped_entrypoint != original, original
        pyproject_path.write_text(escaped_entrypoint, encoding="utf-8")
        escape_failure = run_aw(
            root, "ec", "check", "--project", "demo", expect_success=False
        )
        assert (
            "Python artifact entrypoint must be a safe project-relative path"
            in escape_failure.stderr
        ), escape_failure.stderr
        assert "../escape.py" in escape_failure.stderr, escape_failure.stderr

        # Negative 2: independently, dropping the required uv.lock
        # dependency-file declaration fails closed with the exact protocol
        # error, unrelated to the entrypoint mutation above.
        pyproject_path.write_text(original, encoding="utf-8")
        missing_lock = original.replace(
            'dependency_files = ["pyproject.toml", "uv.lock"]',
            'dependency_files = ["pyproject.toml"]',
            1,
        )
        assert missing_lock != original, original
        pyproject_path.write_text(missing_lock, encoding="utf-8")
        lock_failure = run_aw(root, "ec", "check", "--project", "demo", expect_success=False)
        assert (
            "Python artifact dependency_files must include uv.lock" in lock_failure.stderr
        ), lock_failure.stderr

        # Restore both declarations verbatim: the identical project passes
        # the identical check again, proving the two failures above were
        # caused specifically by the mutated boundary declarations and not
        # by incidental fixture drift.
        pyproject_path.write_text(original, encoding="utf-8")
        restored_check = final_json(run_aw(root, "ec", "check", "--project", "demo", "--json"))
        assert restored_check["clean"] is True, restored_check
        assert restored_check == clean_check, (restored_check, clean_check)

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
