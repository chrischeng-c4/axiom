"""Black-box contract for the total-health-observation-and-readiness-gate
epic: proves the two-cell EC/TD semantic-health model, globally unique
Python TD artifact identity, and authored unit-test ownership for both
executable EC and TD artifacts genuinely hold TOGETHER in one real
`aw health --project <p> spec` observation -- not merely as three
independently true facts asserted by three separate commands, and not a
duplicate of claim #8's own case.

Builds one Python-Spec fixture project (its own TD root, its own EC root,
its own mutation-adequacy declaration) and drives three real phases:

1. baseline -- `td check`, `ec check`, and `aw health ... spec` all pass
   together in one observation, with the spec payload's two semantic-health
   cells independently confirmed passed/aligned via the shared oracle
   helpers `oracles.project_health_contract` already uses for the sibling
   `project-health-total-observation` case.
2. TD artifact-identity break -- a duplicate `__aw_artifact_id__` makes
   `td check` fail and makes the *whole* `aw health ... spec` command fail
   outright (non-zero exit, not just an internal finding), while `ec check`
   on the untouched EC root keeps passing -- proving TD identity uniqueness
   is a genuine causal precondition of the aggregate health observation.
3. TD unit-test-ownership regression, identity restored -- deleting the TD
   root's authored unit test fails `td check` with claim #8's own
   diagnostic while `aw health ... spec` keeps succeeding end to end,
   because the semantic-health evaluator never runs `td check`'s
   unit-test gate -- proving claim #8's axis and this two-cell
   semantic-health axis are structurally independent preconditions, not
   the same underlying check reported under two different commands.
"""

from __future__ import annotations

import hashlib
import json
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from oracles.project_health_contract import assert_alignment, assert_ec_accepts_td
from wi_contract_fixture import (
    final_json,
    run_aw,
    write_python_artifact_lock,
    write_python_artifact_unit_test,
)

CASE_ID = "aw-core-client-total-health-observation-and-readiness-gate"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "total-health-observation-and-readiness-gate"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-total-health-observation-and-readiness-gate"
)
ASSERTIONS = (
    "one real `aw health --project <p> spec` observation reports both "
    "semantic-health cells (`ec_accepts_td` passed with the fixture's one "
    "EC case counted and aligned, `ec_td_alignment` passed with nothing "
    "missing in either direction) at the same time `td check` and "
    "`ec check` independently pass on the same fixture -- the aggregate "
    "promise together, not three separately true facts",
    "introducing a duplicate `__aw_artifact_id__` fails `td check` with "
    "the `duplicate-project-artifact-id` diagnostic and turns the same "
    "real `aw health ... spec` observation non-zero-exit `blocked`, "
    "cascading the identical diagnostic into both semantic-health cells "
    "(`unavailable`) and into the `python_spec` axis's own readiness and "
    "blockers, while `ec check` on the untouched EC root keeps passing -- "
    "proving TD artifact-identity uniqueness is a real, shared causal "
    "precondition of the aggregate health observation, not a coincidence",
    "restoring identity and then deleting the TD root's authored unit "
    "test fails `td check` with claim #8's own 'has no authored unit "
    "tests' diagnostic while `aw health ... spec` keeps succeeding end to "
    "end, proving unit-test ownership and two-cell semantic health are "
    "genuinely independent preconditions rather than the same check "
    "reported under two different command names",
)

_CAPABILITIES = """\
# Fixture Capabilities

## Brief

Fixture health observation contract.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Fixture health | - | implemented | verified | smoke | ready | Total health observation fixture |

### Fixture health

ID: fixture-health
Type: DeveloperTool
Surfaces:
- CLI: `aw health` - observe the fixture project
EC Dimensions:
- behavior: `python3 projects/fixture/external-contracts/src/runner.py --case fixture-health` - health remains observable
Root WI: -
Status: verified
Required Verification: smoke
Promise:
The fixture health request remains externally observable.
Gate Inventory:
- `python3 projects/fixture/external-contracts/src/runner.py --case fixture-health`

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| Fixture health contract | epic | fixture-epic | implemented | verified | smoke | `python3 projects/fixture/external-contracts/src/runner.py --case fixture-health` |
"""

_EC_PYPROJECT = """\
[project]
name = "fixture-external-contracts"
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
author = "agent:fixture-author"
efficiency_policy = "optional"

[[tool.aw.python-ec.cases]]
id = "fixture-health"
artifact_id = "artifact:fixture/fixture-health"
capability_id = "fixture-health"
use_case_id = "fixture-health-contract"
dimension = "behavior"
applicability = "td"
test_path = "src/cases/fixture-health.py"
promise = "The fixture health contract remains observable."
oracle = "The fixture returns one explicit assertion."
target = "rust"
command = "uv run --frozen --offline --project projects/fixture/external-contracts python projects/fixture/external-contracts/src/runner.py --case fixture-health"
evidence_paths = ["evidence/fixture-health.json"]
"""

_DUPLICATE_ARTIFACT_MODULE = """\
__aw_artifact_id__ = "artifact:fixture/fixture-health"


def duplicate_contract() -> bool:
    return True
"""


def _compute_source_digest(ec_root: Path) -> str:
    """Replicate the Rust digest_files algorithm over the fixture EC source_roots=["src"].

    For each file in sorted order: feed relative_path_bytes + NUL +
    len_u64_be + NUL + file_bytes into a single SHA-256 accumulator.
    """
    src_root = ec_root / "src"
    files = sorted(src_root.rglob("*.py"))
    hasher = hashlib.sha256()
    for path in files:
        relative = path.relative_to(ec_root).as_posix().encode()
        content = path.read_bytes()
        hasher.update(relative)
        hasher.update(b"\x00")
        hasher.update(len(content).to_bytes(8, "big"))
        hasher.update(b"\x00")
        hasher.update(content)
    return "sha256:" + hasher.hexdigest()


def _fixture_health_evidence_payload(ec_root: Path) -> dict[str, object]:
    source_digest = _compute_source_digest(ec_root)
    implementation_path = ec_root / "src/cases/fixture-health.py"
    implementation_digest = "sha256:" + hashlib.sha256(
        implementation_path.read_bytes()
    ).hexdigest()
    assertions = ["fixture health"]
    assertions_json = json.dumps(assertions, ensure_ascii=True, separators=(",", ":"))
    assertions_digest = "sha256:" + hashlib.sha256(assertions_json.encode()).hexdigest()
    return {
        "assertions": assertions,
        "attempts": [
            {
                "assertion_count": len(assertions),
                "assertions_digest": assertions_digest,
                "elapsed_ms": 1,
                "exit_code": 0,
            }
        ],
        "case_id": "fixture-health",
        "declared_command": (
            "uv run --frozen --offline --project projects/fixture/external-contracts"
            " python projects/fixture/external-contracts/src/runner.py --case fixture-health"
        ),
        "exit_code": 0,
        "implementation": "src/cases/fixture-health.py",
        "implementation_digest": implementation_digest,
        "mode": "behavior",
        "protocol": "aw.python-ec.evidence.v1",
        "source_digest": source_digest,
        "threshold_seconds": None,
    }


def _write_fixture(root: Path) -> Path:
    (root / "projects/fixture/src").mkdir(parents=True)
    (root / "aw.toml").write_text(
        """\
[[projects]]
name = "fixture"
label = "app:fixture"
path = "projects/fixture"
td_path = "projects/fixture/tech-design"
cap_path = "projects/fixture/CAPABILITIES.md"

[[projects.workspaces]]
name = "fixture"
paths = ["projects/fixture/**"]
target = "rust"
test_cmd = "true"
verify_cold = false
""",
        encoding="utf-8",
    )
    project = root / "projects/fixture"
    (project / "CAPABILITIES.md").write_text(_CAPABILITIES, encoding="utf-8")
    (project / "src/lib.rs").write_text(
        "// HANDWRITE-BEGIN gap=\"fixture\" tracker=\"#fixture\" "
        'reason="fixture generator gap"\n'
        "/// @spec projects/fixture/tech-design/fixture.md#source\n"
        "pub fn fixture() {}\n"
        "// HANDWRITE-END\n",
        encoding="utf-8",
    )

    (project / "tech-design/src").mkdir(parents=True)
    (project / "tech-design/pyproject.toml").write_text(
        '[project]\nname = "fixture-tech-design"\nversion = "0.1.0"\n'
        'requires-python = ">=3.11"\n',
        encoding="utf-8",
    )
    write_python_artifact_lock(project / "tech-design", name="fixture-tech-design")
    (project / "tech-design/src/fixture_health.py").write_text(
        '__aw_artifact_id__ = "artifact:fixture/fixture-health"\n'
        "__aw_public_contract__ = True\n"
        "\n\n"
        "def fixture_health_contract() -> bool:\n"
        "    return True\n",
        encoding="utf-8",
    )
    write_python_artifact_unit_test(project / "tech-design", "fixture_health")

    (project / "external-contracts/src/cases").mkdir(parents=True)
    (project / "external-contracts/evidence").mkdir()
    (project / "external-contracts/src/runner.py").write_text(
        "raise SystemExit(0)\n", encoding="utf-8"
    )
    (project / "external-contracts/src/cases/fixture-health.py").write_text(
        'CASE_ID = "fixture-health"\n\n\ndef verify() -> list[str]:\n'
        '    return ["fixture health"]\n',
        encoding="utf-8",
    )
    (project / "external-contracts/pyproject.toml").write_text(
        _EC_PYPROJECT, encoding="utf-8"
    )
    write_python_artifact_lock(
        project / "external-contracts", name="fixture-external-contracts"
    )
    write_python_artifact_unit_test(project / "external-contracts", "fixture_health")
    ec_root = project / "external-contracts"
    (ec_root / "evidence/fixture-health.json").write_text(
        json.dumps(_fixture_health_evidence_payload(ec_root), indent=2, sort_keys=True)
        + "\n",
        encoding="utf-8",
    )

    (project / "aw.toml").write_text(
        """\
[project]
name = "fixture"
mutation_adequacy = "advisory"
mutation_evidence_dir = "projects/fixture/evidence/mutation-adequacy"
mutation_source_path = "projects/fixture/src"
""",
        encoding="utf-8",
    )
    (project / "evidence").mkdir(parents=True)

    return project


def verify() -> list[str]:
    with tempfile.TemporaryDirectory(prefix="aw-total-health-") as raw_root:
        root = Path(raw_root)
        project = _write_fixture(root)
        td_dup = project / "tech-design/src/fixture_health_duplicate.py"
        td_test = project / "tech-design/tests/unit/test_fixture_health.py"

        # -- phase 1: baseline -- everything passes together -----------------
        td_ok = run_aw(root, "td", "check", "projects/fixture/tech-design", "--project", "fixture")
        assert "Python TD check passed" in td_ok.stdout, td_ok.stdout

        ec_ok = run_aw(root, "ec", "check", "--project", "fixture")
        assert "ec check fixture: clean" in ec_ok.stdout, ec_ok.stdout

        spec = final_json(run_aw(root, "health", "--project", "fixture", "spec"))
        assert spec["status"] == "done", spec
        assert spec["assessment"] == "healthy", spec
        data = spec["data"]
        assert data["python_spec"]["ready"] is True, data
        assert data["mutation"]["required_for_production"] is False, data
        semantic = data["semantic_health"]
        assert_ec_accepts_td(
            semantic["ec_accepts_td"],
            evaluation="passed",
            case_count=1,
            passed_count=1,
            failed_cases=[],
            missing_evidence_cases=[],
        )
        assert_alignment(
            semantic["ec_td_alignment"], missing_in_td=[], missing_in_ec=[]
        )
        assert "mutation" in data, data

        # -- phase 2: duplicate TD artifact id breaks the aggregate ----------
        td_dup.write_text(_DUPLICATE_ARTIFACT_MODULE, encoding="utf-8")

        td_broken = run_aw(
            root,
            "td",
            "check",
            "projects/fixture/tech-design",
            "--project",
            "fixture",
            expect_success=False,
        )
        combined = td_broken.stdout + td_broken.stderr
        assert "duplicate-project-artifact-id" in combined, combined

        ec_still_ok = run_aw(root, "ec", "check", "--project", "fixture")
        assert "ec check fixture: clean" in ec_still_ok.stdout, ec_still_ok.stdout

        broken_health = run_aw(
            root, "health", "--project", "fixture", "spec", expect_success=False
        )
        assert broken_health.returncode != 0, broken_health
        broken_payload = final_json(broken_health)
        assert broken_payload["status"] == "blocked", broken_payload
        assert broken_payload["assessment"] == "blocked", broken_payload
        assert broken_payload["section"] == "spec", broken_payload
        broken_data = broken_payload["data"]
        broken_accepts = broken_data["semantic_health"]["ec_accepts_td"]
        assert broken_accepts["evaluation"] == "unavailable", broken_payload
        assert broken_accepts["case_count"] == 0, broken_payload
        assert any(
            "duplicate-project-artifact-id" in finding
            for finding in broken_accepts["findings"]
        ), broken_payload
        assert (
            broken_data["semantic_health"]["ec_td_alignment"]["evaluation"]
            == "unavailable"
        ), broken_payload
        assert broken_data["python_spec"]["ready"] is False, broken_payload
        assert any(
            "duplicate-project-artifact-id" in blocker
            for blocker in broken_data["python_spec"]["blockers"]
        ), broken_payload

        # -- phase 3: identity restored, TD unit-test ownership regresses ---
        td_dup.unlink()
        td_test.unlink()

        td_no_tests = run_aw(
            root,
            "td",
            "check",
            "projects/fixture/tech-design",
            "--project",
            "fixture",
            expect_success=False,
        )
        combined = td_no_tests.stdout + td_no_tests.stderr
        assert "has no authored unit tests" in combined, combined
        assert "duplicate-project-artifact-id" not in combined, combined

        healthy_again = final_json(
            run_aw(root, "health", "--project", "fixture", "spec")
        )
        assert healthy_again["status"] == "done", healthy_again
        assert healthy_again["data"]["python_spec"]["ready"] is True, healthy_again

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
