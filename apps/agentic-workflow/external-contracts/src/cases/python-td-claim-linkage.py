"""Black-box contract for Python TD capability claim linkage."""

from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
import tempfile
from pathlib import Path

from wi_contract_fixture import (
    write_python_artifact_lock,
    write_python_artifact_unit_test,
)


CASE_ID = "python-td-claim-linkage"
CAPABILITY_ID = "capability-control-plane"
USE_CASE_ID = "python-td-claim-linkage"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case python-td-claim-linkage"
)
ASSERTIONS = (
    "a matching Python TD artifact and public behavior becomes primary claim evidence",
    "the evidence names the Python TD source and explicit artifact identity",
    "the linked claim routes exactly to runtime verification without a WI plan",
    "a legacy Markdown capability_refs decoy is ignored in Python TD mode",
    "an unmatched Python TD behavior creates no claim evidence and routes to linkage remediation",
    "a wrong Python TD artifact with the matching behavior creates no claim evidence",
    "a missing Python TD manifest fails closed instead of accepting legacy Markdown evidence",
    "a missing Python TD root fails closed before any legacy compatibility path",
)


def _repository_root() -> Path:
    return Path(__file__).resolve().parents[5]


def _aw_binary() -> Path:
    return _repository_root() / "target" / "debug" / "aw"


def _capability_document() -> str:
    return """# Demo Capabilities

## Brief

Isolated Python TD claim-linkage fixture.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Demo | - | implemented | verified | smoke | ready | Python TD and EC edges are linked |

### Demo

ID: demo-capability
Type: DeveloperTool
Surfaces: CLI: `aw capability report --project demo` - reports capability evidence.
EC Dimensions: behavior: `true` - isolated black-box claim-linkage contract.
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Expose one Python-authored behavior with primary TD claim evidence.
Gate Inventory:
- `true`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Python TD claim linkage | change | - | implemented | verified | smoke | `true` |
"""


def _compute_source_digest(ec_root: Path) -> str:
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


def _write_canonical_evidence(ec_root: Path) -> None:
    source_digest = _compute_source_digest(ec_root)
    implementation_digest = "sha256:" + hashlib.sha256(
        (ec_root / "src/cases/claim.py").read_bytes()
    ).hexdigest()
    evidence = {
        "protocol": "aw.python-ec.evidence.v1",
        "case_id": "python-td-claim-linkage",
        "mode": "behavior",
        "source_digest": source_digest,
        "declared_command": "true",
        "implementation": "src/cases/claim.py",
        "implementation_digest": implementation_digest,
        "exit_code": 0,
        "assertions": ["claim is externally observable"],
        "attempts": [{"exit_code": 0, "assertion_count": 1}],
    }
    (ec_root / "evidence/claim.json").write_text(
        json.dumps(evidence, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


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
cap_path = "project/README.md"
label = "app:demo"

[[projects.workspaces]]
name = "demo"
paths = ["project/**"]
target = "python"
test_cmd = "true"
""",
        encoding="utf-8",
    )
    (project / "README.md").write_text(_capability_document(), encoding="utf-8")
    (td_root / "pyproject.toml").write_text(
        """[project]
name = "demo-tech-design"
version = "0.1.0"
requires-python = ">=3.11"
""",
        encoding="utf-8",
    )
    (td_root / "src/demo/public_contracts/claim_linkage.py").write_text(
        '''__aw_artifact_id__ = "artifact:demo/claim-linkage"
__aw_public_contract__ = True


def python_td_claim_linkage() -> str:
    return "Python TD claim linkage"
''',
        encoding="utf-8",
    )
    (td_root / "legacy-decoy.md").write_text(
        """---
id: legacy-decoy
capability_refs:
  - id: demo-capability
    role: primary
    claim: python-td-claim-linkage
    coverage: full
    rationale: "This legacy decoy must be ignored by Python TD linkage."
---

# Legacy decoy
""",
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
id = "python-td-claim-linkage"
artifact_id = "artifact:demo/claim-linkage"
capability_id = "demo-capability"
use_case_id = "python-td-claim-linkage"
dimension = "behavior"
applicability = "td"
test_path = "src/cases/claim.py"
promise = "the capability report exposes the matching Python TD edge"
oracle = "the outer EC independently inspects the real aw process output"
target = "python"
command = "true"
evidence_paths = ["evidence/claim.json"]
""",
        encoding="utf-8",
    )
    write_python_artifact_lock(ec_root, name="demo-external-contracts")
    write_python_artifact_unit_test(ec_root, "claim_linkage")
    _write_canonical_evidence(ec_root)


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
    with tempfile.TemporaryDirectory(prefix="aw-python-claim-linkage-") as raw_tmp:
        root = Path(raw_tmp)
        _write_fixture(root)

        # Negative control / falsifier: incomplete old evidence yields unready status
        # and routes to run_verify.
        (root / "project/external-contracts/evidence/claim.json").write_text(
            '{"protocol":"aw.python-ec.evidence.v1","exit_code":0}\n',
            encoding="utf-8",
        )
        unready_report = _report(root)
        assert unready_report["python_artifact"]["ready"] is False, unready_report
        assert unready_report["next_action"]["kind"] == "run_verify", unready_report

        # Restore canonical evidence
        _write_canonical_evidence(root / "project/external-contracts")
        report = _report(root)
        capability = report["capabilities"][0]
        td_refs = capability["td_refs"]
        assert len(td_refs) == 1, td_refs
        td_ref = td_refs[0]
        assert td_ref["role"] == "primary", td_ref
        assert td_ref["claim"] == "python-td-claim-linkage", td_ref
        assert td_ref["spec_id"] == "artifact:demo/claim-linkage", td_ref
        assert td_ref["spec_path"].endswith(
            "tech-design/src/demo/public_contracts/claim_linkage.py"
        ), td_ref
        assert report["next_action"]["kind"] == "run_verify", report
        assert report["next_action"]["command"] == (
            "aw capability report --project demo --verify --skip-issue-inventory"
        ), report
        assert "legacy-decoy.md" not in td_ref["spec_path"], td_ref

        td_contract = (
            root
            / "project/tech-design/src/demo/public_contracts/claim_linkage.py"
        )
        td_contract.write_text(
            '''__aw_artifact_id__ = "artifact:demo/claim-linkage"
__aw_public_contract__ = True


def unrelated_behavior() -> str:
    return "This behavior does not match the EC use-case identity"
''',
            encoding="utf-8",
        )
        unmatched = _report(root)
        unmatched_capability = unmatched["capabilities"][0]
        assert unmatched_capability["td_refs"] == [], unmatched_capability
        assert (
            unmatched["next_action"]["kind"] == "link_claim_verification"
        ), unmatched
        assert unmatched["next_action"]["command"] == "aw wi plan --project demo"

        td_contract.write_text(
            '''__aw_artifact_id__ = "artifact:demo/wrong-artifact"
__aw_public_contract__ = True


def python_td_claim_linkage() -> str:
    return "The behavior matches but the artifact identity does not"
''',
            encoding="utf-8",
        )
        wrong_artifact = _report(root)
        wrong_artifact_capability = wrong_artifact["capabilities"][0]
        assert wrong_artifact_capability["td_refs"] == [], wrong_artifact_capability
        assert (
            wrong_artifact["next_action"]["kind"] == "link_claim_verification"
        ), wrong_artifact

        td_contract.write_text(
            '''__aw_artifact_id__ = "artifact:demo/claim-linkage"
__aw_public_contract__ = True


def python_td_claim_linkage() -> str:
    return "Python TD claim linkage"
''',
            encoding="utf-8",
        )
        td_manifest = root / "project/tech-design/pyproject.toml"
        td_manifest.rename(td_manifest.with_suffix(".missing"))
        missing_manifest = _report(root)
        missing_capability = missing_manifest["capabilities"][0]
        assert missing_capability["td_refs"] == [], missing_capability
        assert any(
            blocker.startswith("td capability scan unavailable:")
            and "pyproject.toml" in blocker
            for blocker in missing_manifest["blockers"]
        ), missing_manifest
        assert missing_manifest["next_action"]["kind"] != "run_verify", missing_manifest

        shutil.rmtree(root / "project/tech-design")
        missing_root = _report(root)
        missing_root_capability = missing_root["capabilities"][0]
        assert missing_root_capability["td_refs"] == [], missing_root_capability
        assert any(
            blocker.startswith("td capability scan unavailable:")
            and "pyproject.toml" in blocker
            for blocker in missing_root["blockers"]
        ), missing_root
        assert missing_root["status"] == "blocked", missing_root
        assert missing_root["next_action"]["command"].startswith(
            "aw td check "
        ), missing_root
        assert missing_root["next_action"]["reason"] == (
            "Python artifact inventory or digest-bound evidence is not ready"
        ), missing_root

    return list(ASSERTIONS)
