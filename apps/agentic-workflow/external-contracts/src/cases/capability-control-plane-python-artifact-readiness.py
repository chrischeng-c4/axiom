"""Black-box contract for the shared Python TD/EC readiness projection (#2304)."""

from __future__ import annotations

import hashlib
import json
import os
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
    "executing a non-empty verifier writes canonical current-case evidence that clears the blocker and flips readiness to ready in both surfaces",
    "malformed, unsupported-protocol, wrong-case, stale-source, stale-implementation, mismatched-command, non-zero-exit, and zero-assertion evidence each fail closed with an exact blocker",
)

DECLARED_COMMAND = (
    "uv run --frozen --offline --project . python "
    "src/runner.py --case demo-readiness"
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
        '''from __future__ import annotations

import hashlib
import json
import os
import runpy
import sys
from pathlib import Path


CASE_ID = "demo-readiness"
DECLARED_COMMAND = (
    "uv run --frozen --offline --project . python "
    "src/runner.py --case demo-readiness"
)


def digest_bytes(value: bytes) -> str:
    return "sha256:" + hashlib.sha256(value).hexdigest()


def main() -> int:
    if sys.argv[1:] != ["--case", CASE_ID]:
        raise RuntimeError(f"expected --case {CASE_ID}")
    root = Path(__file__).resolve().parents[1]
    implementation = root / "src/cases/readiness.py"
    verifier = runpy.run_path(str(implementation))["verify"]
    assertions = verifier()
    if not assertions:
        raise RuntimeError("fixture verifier executed zero assertions")
    evidence = {
        "protocol": "aw.python-ec.evidence.v1",
        "case_id": CASE_ID,
        "mode": "behavior",
        "source_digest": os.environ["AW_PYTHON_EC_SOURCE_DIGEST"],
        "declared_command": DECLARED_COMMAND,
        "implementation": "src/cases/readiness.py",
        "implementation_digest": digest_bytes(implementation.read_bytes()),
        "exit_code": 0,
        "assertions": assertions,
        "attempts": [
            {
                "exit_code": 0,
                "assertion_count": len(assertions),
            }
        ],
    }
    evidence_path = root / "evidence/readiness.json"
    evidence_path.write_text(
        json.dumps(evidence, indent=2, sort_keys=True) + "\\n",
        encoding="utf-8",
    )
    print(json.dumps(evidence, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
''',
        encoding="utf-8",
    )
    (ec_root / "src/cases/readiness.py").write_text(
        'def verify() -> list[str]:\n'
        '    return ["readiness is externally observable"]\n',
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
command = "uv run --frozen --offline --project . python src/runner.py --case demo-readiness"
evidence_paths = ["evidence/readiness.json"]
""",
        encoding="utf-8",
    )
    write_python_artifact_lock(ec_root, name="demo-external-contracts")
    write_python_artifact_unit_test(ec_root, "readiness")
    evidence_path = ec_root / "evidence/readiness.json"
    if evidence_path.exists():
        evidence_path.unlink()
    if with_evidence:
        _execute_fixture_verifier(root)


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


def _execute_fixture_verifier(root: Path) -> dict[str, object]:
    ec_root = root / "project/external-contracts"
    before = _report(root)["python_artifact"]
    source_digest = before["ec_source_digest"]
    assert isinstance(source_digest, str) and source_digest.startswith("sha256:"), before
    independently_computed = _independent_source_digest(ec_root)
    assert source_digest == independently_computed, (source_digest, independently_computed)
    env = os.environ.copy()
    env["AW_PYTHON_EC_SOURCE_DIGEST"] = independently_computed
    completed = subprocess.run(
        [
            "uv",
            "run",
            "--frozen",
            "--offline",
            "--project",
            ".",
            "python",
            "src/runner.py",
            "--case",
            "demo-readiness",
        ],
        cwd=ec_root,
        env=env,
        capture_output=True,
        text=True,
        check=False,
    )
    assert completed.returncode == 0, (completed.stdout, completed.stderr)
    evidence = json.loads((ec_root / "evidence/readiness.json").read_text())
    assert evidence["case_id"] == "demo-readiness", evidence
    assert evidence["source_digest"] == source_digest, evidence
    assert evidence["declared_command"] == DECLARED_COMMAND, evidence
    assert evidence["assertions"] == ["readiness is externally observable"], evidence
    assert evidence["attempts"] == [{"assertion_count": 1, "exit_code": 0}], evidence
    return evidence


def _independent_source_digest(ec_root: Path) -> str:
    ignored = {
        "__pycache__",
        ".venv",
        "venv",
        ".pytest_cache",
        ".mypy_cache",
        ".ruff_cache",
        ".tox",
        "build",
        "dist",
        ".eggs",
    }
    files = sorted(
        path
        for path in (ec_root / "src").rglob("*.py")
        if not any(part in ignored for part in path.relative_to(ec_root).parts)
    )
    assert files, ec_root
    digest = hashlib.sha256()
    for path in files:
        relative = path.relative_to(ec_root).as_posix().encode()
        body = path.read_bytes()
        digest.update(relative)
        digest.update(b"\0")
        digest.update(len(body).to_bytes(8, byteorder="big"))
        digest.update(b"\0")
        digest.update(body)
    return "sha256:" + digest.hexdigest()


def _assert_invalid_evidence(
    root: Path,
    body: str,
    *,
    blocker: str,
) -> None:
    evidence_path = root / "project/external-contracts/evidence/readiness.json"
    evidence_path.write_text(body, encoding="utf-8")
    report = _report(root)
    health = _health_spec(root)
    readiness = report["python_artifact"]
    assert readiness == health["data"]["python_spec"], (readiness, health)
    assert readiness["ready"] is False, readiness
    assert readiness["ready_case_count"] == 0, readiness
    assert readiness["cases"][0]["evidence_ready"] is False, readiness
    assert readiness["blockers"] == [blocker], readiness
    assert readiness["next_command"] == "aw ec verify --project demo --stage td", (
        readiness
    )


def _mutated_evidence(
    evidence: dict[str, object],
    *path_and_value: tuple[tuple[str | int, ...], object],
) -> str:
    value = json.loads(json.dumps(evidence))
    for path, replacement in path_and_value:
        cursor = value
        for key in path[:-1]:
            cursor = cursor[key]
        cursor[path[-1]] = replacement
    return json.dumps(value, indent=2, sort_keys=True) + "\n"


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

        valid_evidence = json.loads(
            (
                root
                / "project/external-contracts/evidence/readiness.json"
            ).read_text(encoding="utf-8")
        )
        invalid_prefix = (
            "Python EC case `demo-readiness` evidence "
            "`evidence/readiness.json`"
        )
        _assert_invalid_evidence(
            root,
            "{not-json\n",
            blocker=f"{invalid_prefix} is not valid JSON",
        )
        _assert_invalid_evidence(
            root,
            _mutated_evidence(
                valid_evidence,
                (("protocol",), "aw.python-ec.evidence.v0"),
            ),
            blocker=f"{invalid_prefix} has unsupported protocol",
        )
        _assert_invalid_evidence(
            root,
            _mutated_evidence(
                valid_evidence,
                (("case_id",), "another-case"),
            ),
            blocker=f"{invalid_prefix} names case `another-case`",
        )
        implementation_path = (
            root / "project/external-contracts/src/cases/readiness.py"
        )
        original_implementation = implementation_path.read_text(encoding="utf-8")
        implementation_path.write_text(
            original_implementation + "\n# source drift\n", encoding="utf-8"
        )
        _assert_invalid_evidence(
            root,
            json.dumps(valid_evidence, indent=2, sort_keys=True) + "\n",
            blocker=f"{invalid_prefix} is stale for the current source digest",
        )

        current_source_digest = _independent_source_digest(
            root / "project/external-contracts"
        )
        _assert_invalid_evidence(
            root,
            _mutated_evidence(
                valid_evidence,
                (("source_digest",), current_source_digest),
            ),
            blocker=f"{invalid_prefix} is stale for `src/cases/readiness.py`",
        )
        implementation_path.write_text(original_implementation, encoding="utf-8")

        inventory_path = root / "project/external-contracts/pyproject.toml"
        original_inventory = inventory_path.read_text(encoding="utf-8")
        inventory_path.write_text(
            original_inventory.replace(
                f'command = "{DECLARED_COMMAND}"',
                f'command = "{DECLARED_COMMAND} --strict"',
            ),
            encoding="utf-8",
        )
        _assert_invalid_evidence(
            root,
            json.dumps(valid_evidence, indent=2, sort_keys=True) + "\n",
            blocker=f"{invalid_prefix} does not match the declared command",
        )
        inventory_path.write_text(original_inventory, encoding="utf-8")
        _assert_invalid_evidence(
            root,
            _mutated_evidence(
                valid_evidence,
                (("exit_code",), 1),
                (("attempts", 0, "exit_code"), 1),
            ),
            blocker=f"{invalid_prefix} does not record successful execution",
        )
        _assert_invalid_evidence(
            root,
            _mutated_evidence(
                valid_evidence,
                (("assertions",), []),
                (("attempts", 0, "assertion_count"), 0),
            ),
            blocker=f"{invalid_prefix} records zero executed assertions or tests",
        )

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
