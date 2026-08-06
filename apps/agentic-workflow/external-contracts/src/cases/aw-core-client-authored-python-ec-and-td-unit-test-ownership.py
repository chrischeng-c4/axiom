"""Black-box contract for independent authored-unit-test ownership on the
Python TD and EC artifact roots.

Drives the real `aw td check` and `aw ec check` structural checks against a
fixture project with both a Python TD root and a Python EC root, proving
each root requires and *runs* its own `tests/unit/test_*.py` inventory
independently of the other: deleting one root's authored unit tests flags
only that root while the sibling root stays green, and swapping a present
test file's body for a failing assertion produces a distinct "tests failed"
diagnostic instead of the "no authored unit tests" one -- proving the gate
actually executes the suite rather than merely checking for its presence on
disk.
"""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from wi_contract_fixture import (
    project_fixture,
    run_aw,
    write_python_artifact_lock,
    write_python_artifact_unit_test,
)

CASE_ID = "aw-core-client-authored-python-ec-and-td-unit-test-ownership"
CAPABILITY_ID = "aw-core-client-model-workitem-first-artifact-lifecycle"
USE_CASE_ID = "authored-python-ec-and-td-unit-test-ownership"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "uv run --frozen --offline --project apps/agentic-workflow/external-contracts "
    "python apps/agentic-workflow/external-contracts/src/runner.py "
    "--case aw-core-client-authored-python-ec-and-td-unit-test-ownership"
)
ASSERTIONS = (
    "`aw td check <td-root> --project <p>` and `aw ec check --project <p>` "
    "both pass and report exactly one authored unit-test file each when "
    "their respective TD and EC roots carry a real `tests/unit/test_*.py`",
    "deleting the TD root's authored unit test flags only `td check` with "
    "'has no authored unit tests' while `ec check` on the untouched EC root "
    "keeps passing, and the same independence holds in reverse for a "
    "deleted EC unit test against an unaffected, still-passing `td check`",
    "once a deliberately failing (not missing) unit test replaces an "
    "authored test file's body, both `td check` and `ec check` fail with "
    "the distinct 'authored Python unit tests failed' diagnostic rather "
    "than the 'no authored unit tests' one, proving each root's inventory "
    "is actually executed rather than merely checked for presence on disk",
)

_TD_ARTIFACT = """\
__aw_artifact_id__ = "artifact:demo/demo"
__aw_public_contract__ = True


def demo_contract() -> bool:
    return True
"""

_EC_PYPROJECT = """\
[project]
name = "demo-external-contracts"
version = "0.0.0"
requires-python = ">=3.11"

[tool.aw.python-artifact]
protocol = "aw.python-artifact.v1"
entrypoint = "src/runner.py"
source_roots = ["src"]
dependency_files = ["pyproject.toml", "uv.lock"]
evidence_dir = "evidence"

[tool.aw.python-ec]
protocol = "aw.python-ec.v1"
author = "fixture-author"
efficiency_policy = "optional"

[[tool.aw.python-ec.cases]]
id = "demo-contract"
artifact_id = "artifact:demo/demo"
capability_id = "demo"
use_case_id = "demo-contract"
dimension = "behavior"
applicability = "td"
test_path = "src/demo_contract.py"
promise = "The demo contract stays observable."
oracle = "The public EC checker reports its structural inventory."
target = "rust"
command = "uv run --frozen --offline --project external-contracts python -c 'print(\\\"demo\\\")'"
evidence_paths = ["evidence/demo-contract.json"]
"""

_CAPABILITIES = """\
# Demo Capabilities

## Brief

Demo capability contract for authored unit-test ownership.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Demo | - | implemented | verified | smoke | ready | Authored unit-test ownership |

### Demo

ID: demo
Type: DeveloperTool
Surfaces:
- CLI: demo
EC Dimensions:
- behavior: demo contract stays observable
Root WI: -
Status: verified
Required Verification: smoke
Promise:
The demo contract stays observable.
Gate Inventory:
- external-contracts/evidence/demo-contract.json
"""

_PASSING_TEST_BODY = (
    "import unittest\n\n\nclass OwnershipTest(unittest.TestCase):\n"
    "    def test_passes(self) -> None:\n"
    "        self.assertTrue(True)\n"
)
_FAILING_TEST_BODY = (
    "import unittest\n\n\nclass OwnershipTest(unittest.TestCase):\n"
    "    def test_fails(self) -> None:\n"
    "        self.assertEqual(1, 2)\n"
)


def _write_td_root(root: Path) -> Path:
    td_root = root / "tech-design"
    (td_root / "src").mkdir(parents=True)
    (td_root / "pyproject.toml").write_text(
        '[project]\nname = "demo-tech-design"\nversion = "0.1.0"\n'
        'requires-python = ">=3.11"\n',
        encoding="utf-8",
    )
    write_python_artifact_lock(td_root, name="demo-tech-design")
    (td_root / "src" / "demo.py").write_text(_TD_ARTIFACT, encoding="utf-8")
    write_python_artifact_unit_test(td_root, "td")
    return td_root


def _write_ec_root(root: Path) -> Path:
    ec_root = root / "external-contracts"
    (ec_root / "src").mkdir(parents=True)
    (ec_root / "evidence").mkdir(parents=True)
    (ec_root / "pyproject.toml").write_text(_EC_PYPROJECT, encoding="utf-8")
    write_python_artifact_lock(ec_root, name="demo-external-contracts", version="0.0.0")
    (ec_root / "src" / "runner.py").write_text(
        'print("fixture runner")\n', encoding="utf-8"
    )
    (ec_root / "src" / "demo_contract.py").write_text(
        '"""Declared contract source."""\n', encoding="utf-8"
    )
    (ec_root / "evidence" / "demo-contract.json").write_text(
        '{"status": "passed"}\n', encoding="utf-8"
    )
    write_python_artifact_unit_test(ec_root, "ec")
    return ec_root


def verify() -> list[str]:
    with project_fixture() as root:
        (root / "CAPABILITIES.md").write_text(_CAPABILITIES, encoding="utf-8")
        td_root = _write_td_root(root)
        ec_root = _write_ec_root(root)
        td_test = td_root / "tests/unit/test_td.py"
        ec_test = ec_root / "tests/unit/test_ec.py"

        # -- phase A: both roots green with exactly one authored test each --
        td_ok = run_aw(root, "td", "check", "tech-design", "--project", "demo")
        assert "Python TD check passed" in td_ok.stdout, td_ok.stdout
        assert "1 authored unit-test file(s)" in td_ok.stdout, td_ok.stdout

        ec_ok = run_aw(root, "ec", "check", "--project", "demo")
        assert "ec check demo: clean" in ec_ok.stdout, ec_ok.stdout
        assert "1 authored unit-test file(s)" in ec_ok.stdout, ec_ok.stdout

        # -- phase B: TD unit test missing, EC unaffected --------------------
        td_test.unlink()
        td_missing = run_aw(
            root, "td", "check", "tech-design", "--project", "demo", expect_success=False
        )
        combined = td_missing.stdout + td_missing.stderr
        assert "has no authored unit tests" in combined, combined
        assert "tests/unit/test_*.py" in combined, combined

        ec_unaffected = run_aw(root, "ec", "check", "--project", "demo")
        assert "ec check demo: clean" in ec_unaffected.stdout, ec_unaffected.stdout

        # -- phase C: TD unit test present but deliberately failing ---------
        td_test.write_text(_FAILING_TEST_BODY, encoding="utf-8")
        td_failing = run_aw(
            root, "td", "check", "tech-design", "--project", "demo", expect_success=False
        )
        combined = td_failing.stdout + td_failing.stderr
        assert "authored Python unit tests failed" in combined, combined
        assert "has no authored unit tests" not in combined, combined

        td_test.write_text(_PASSING_TEST_BODY, encoding="utf-8")
        td_restored = run_aw(root, "td", "check", "tech-design", "--project", "demo")
        assert "Python TD check passed" in td_restored.stdout, td_restored.stdout

        # -- phase D: EC unit test missing, TD unaffected --------------------
        ec_test.unlink()
        ec_missing = run_aw(
            root, "ec", "check", "--project", "demo", expect_success=False
        )
        combined = ec_missing.stdout + ec_missing.stderr
        assert "has no authored unit tests" in combined, combined

        td_unaffected = run_aw(root, "td", "check", "tech-design", "--project", "demo")
        assert "Python TD check passed" in td_unaffected.stdout, td_unaffected.stdout

        # -- phase E: EC unit test present but deliberately failing ---------
        ec_test.write_text(_FAILING_TEST_BODY, encoding="utf-8")
        ec_failing = run_aw(
            root, "ec", "check", "--project", "demo", expect_success=False
        )
        combined = ec_failing.stdout + ec_failing.stderr
        assert "authored Python unit tests failed" in combined, combined
        assert "has no authored unit tests" not in combined, combined

    return list(ASSERTIONS)


if __name__ == "__main__":
    verify()
