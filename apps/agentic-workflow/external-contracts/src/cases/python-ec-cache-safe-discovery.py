"""Black-box cache-safe Python EC discovery contract."""

from __future__ import annotations

import json
from pathlib import Path

from wi_contract_fixture import final_json, project_fixture, run_aw


CASE_ID = "python-ec-cache-safe-discovery"
CAPABILITY_ID = "project-local-td-and-ec-gates"
USE_CASE_ID = "cache-safe-python-ec-source-discovery"
DIMENSION = "behavior"
TARGET_COMMAND = (
    "python3 apps/agentic-workflow/external-contracts/src/runner.py "
    "--case python-ec-cache-safe-discovery"
)
ASSERTIONS = (
    "public EC check ignores adjacent Python cache and binary build artifacts",
    "public EC check still reports a missing declared Python contract source",
    "public EC check rejects a binary cache artifact declared as a contract source",
)


PYPROJECT = """\
[project]
name = "demo-external-contracts"
version = "0.0.0"

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
id = "cache-safe-contract"
artifact_id = "artifact:demo/cache-safe-contract"
capability_id = "demo"
use_case_id = "cache-safe-contract"
dimension = "behavior"
applicability = "td"
test_path = "src/cache_safe_contract.py"
promise = "Runtime cache artifacts cannot poison EC source discovery."
oracle = "The public EC checker reports its structural inventory."
target = "rust"
command = "python3 -c 'print(\\\"cache-safe\\\")'"
evidence_paths = ["evidence/cache-safe-contract.json"]
"""


CAPABILITIES = """\
# Demo Capabilities

## Brief

Demo capability contract for Python EC cache discovery.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
| Demo | - | implemented | verified | smoke | ready | Cache-safe EC discovery |

### Demo

ID: demo
Type: DeveloperTool
Surfaces:
- CLI: demo
EC Dimensions:
- behavior: cache-safe EC discovery
Root WI: -
Status: verified
Required Verification: smoke
Promise:
Runtime cache files do not change the declared Python EC source inventory.
Gate Inventory:
- external-contracts/evidence/cache-safe-contract.json
"""


def _write_fixture(root: Path) -> Path:
    ec_root = root / "external-contracts"
    source_root = ec_root / "src"
    evidence_root = ec_root / "evidence"
    source_root.mkdir(parents=True)
    evidence_root.mkdir(parents=True)

    (root / "CAPABILITIES.md").write_text(CAPABILITIES, encoding="utf-8")
    (ec_root / "pyproject.toml").write_text(PYPROJECT, encoding="utf-8")
    (source_root / "runner.py").write_text(
        'print("fixture runner")\n', encoding="utf-8"
    )
    contract = source_root / "cache_safe_contract.py"
    contract.write_text(
        '"""Declared contract source."""\n', encoding="utf-8"
    )
    (evidence_root / "cache-safe-contract.json").write_text(
        json.dumps({"status": "passed"}) + "\n", encoding="utf-8"
    )

    return contract


def _write_runtime_artifacts(root: Path) -> dict[Path, bytes]:
    source_root = root / "external-contracts" / "src"
    artifacts = {
        source_root
        / "__pycache__"
        / "cache_safe_contract.cpython-312.pyc": b"\x00\xff\xfe\x80AW-EC-BEGIN",
        source_root / "native-extension.so": b"\x7fELF\x00\xff\xfeAW-EC-BEGIN",
        source_root / "build" / "generated.py": b"\x00\xff\xfeBUILD-PYTHON",
        source_root / "build" / "manifest.txt": b"\x00\xff\xfeBUILD-TEXT",
        source_root / "build" / "opaque-cache": b"\x00\xff\xfeBUILD-OPAQUE",
    }
    for path, content in artifacts.items():
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
    return artifacts


def verify() -> list[str]:
    with project_fixture() as root:
        contract = _write_fixture(root)

        baseline = final_json(
            run_aw(root, "ec", "check", "--project", "demo", "--json")
        )
        assert baseline["clean"] is True, baseline
        assert baseline["case_count"] == 1, baseline

        artifacts = _write_runtime_artifacts(root)
        pyc = next(path for path in artifacts if path.suffix == ".pyc")
        clean = final_json(
            run_aw(root, "ec", "check", "--project", "demo", "--json")
        )
        assert clean["clean"] is True, clean
        assert clean["case_count"] == 1, clean
        assert clean["findings"] == [], clean
        assert (
            clean["generated_from_td_digest"]
            == baseline["generated_from_td_digest"]
        ), (baseline, clean)

        contract.unlink()
        missing = final_json(
            run_aw(
                root,
                "ec",
                "check",
                "--project",
                "demo",
                "--json",
                expect_success=False,
            )
        )
        assert missing["clean"] is False, missing
        assert any(
            "`test_path` does not exist: `src/cache_safe_contract.py`" in finding
            for finding in missing["findings"]
        ), missing

        contract.write_text(
            '"""Restored declared contract source."""\n', encoding="utf-8"
        )
        inventory = root / "external-contracts" / "pyproject.toml"
        inventory.write_text(
            inventory.read_text(encoding="utf-8").replace(
                'test_path = "src/cache_safe_contract.py"',
                'test_path = "src/__pycache__/cache_safe_contract.cpython-312.pyc"',
            ),
            encoding="utf-8",
        )
        declared_binary = final_json(
            run_aw(
                root,
                "ec",
                "check",
                "--project",
                "demo",
                "--json",
                expect_success=False,
            )
        )
        assert declared_binary["clean"] is False, declared_binary
        assert any(
            "`test_path` must be a safe project-relative src/*.py path" in finding
            and pyc.name in finding
            for finding in declared_binary["findings"]
        ), declared_binary
        for path, expected in artifacts.items():
            assert path.is_file(), path
            assert path.read_bytes() == expected, path

    return list(ASSERTIONS)
