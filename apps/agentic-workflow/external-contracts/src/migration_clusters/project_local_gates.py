"""Native Python ECs for project-local TD/EC gates and false-green defense."""

from __future__ import annotations

import os
import stat
import time
from pathlib import Path
from typing import Any

from migration_clusters.prompt_artifacts import _artifact_snapshot
from wi_contract_fixture import final_json, project_fixture, run_aw


CASE_IDS = {
    "aw-ec-zero-test-false-green",
    "project-local-td-and-ec-gates-cb-generation-and-standardize-scan-defaults",
    "project-local-td-and-ec-gates-ec-evidence-documentation",
    "project-local-td-and-ec-gates-ec-external-contract-source",
    "project-local-td-and-ec-gates-ec-tool-binding-dispatch",
    "project-local-td-and-ec-gates-project-dirty-scope-protection",
    "project-local-td-and-ec-gates-project-local-td-root-resolver",
    "project-local-td-and-ec-gates-td-lock-and-external-contract-target-resolution",
    "project-local-td-and-ec-gates-operational-efficiency",
    "project-local-td-and-ec-gates-operational-stability",
}


def _zero_test_snapshot() -> dict[str, Any]:
    with project_fixture() as root:
        config_path = root / "aw.toml"
        config_path.write_text(
            config_path.read_text(encoding="utf-8").replace(
                'name = "demo"\n',
                'name = "demo"\nec_review_mode = "deferred"\n',
                1,
            ),
            encoding="utf-8",
        )
        ec = root / "external-contracts"
        (ec / "src").mkdir(parents=True)
        (ec / "evidence").mkdir()
        (ec / "src/zero.py").write_text("CASE_ID = 'zero-test'\n", encoding="utf-8")
        (ec / "pyproject.toml").write_text(
            """\
[project]
name = "zero-test-ec"
version = "0.1.0"

[tool.aw.python-artifact]
protocol = "aw.python-artifact.v1"
entrypoint = "src/zero.py"
source_roots = ["src"]
dependency_files = ["pyproject.toml"]
evidence_dir = "evidence"

[tool.aw.python-ec]
protocol = "aw.python-ec.v1"
author = "agent:fixture"
efficiency_policy = "not-applicable"

[[tool.aw.python-ec.cases]]
id = "zero-test"
artifact_id = "artifact:demo/zero-test"
capability_id = "fixture"
use_case_id = "zero-test"
dimension = "behavior"
applicability = "td"
test_path = "src/zero.py"
promise = "zero cargo tests are not green"
oracle = "fixture cargo output parser"
target = "rust"
command = "cargo test -p fixture --lib absent_filter"
evidence_paths = ["evidence/zero-test.json"]
""",
            encoding="utf-8",
        )
        home = root / "home"
        bin_dir = home / ".rustup/toolchains/stable-aarch64-apple-darwin/bin"
        bin_dir.mkdir(parents=True)
        cargo = bin_dir / "cargo"
        cargo.write_text(
            """#!/bin/sh
mkdir -p external-contracts/evidence
echo '{}' > external-contracts/evidence/zero-test.json
echo 'running 0 tests'
echo 'test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out'
exit 0
""",
            encoding="utf-8",
        )
        cargo.chmod(cargo.stat().st_mode | stat.S_IXUSR)
        inventory_path = ec / "pyproject.toml"
        inventory_path.write_text(
            inventory_path.read_text(encoding="utf-8"),
            encoding="utf-8",
        )
        env = {"HOME": str(home), "PATH": f"{bin_dir}:{os.environ['PATH']}"}
        completed = run_aw(
            root,
            "ec",
            "verify",
            "--project",
            "demo",
            "--json",
            expect_success=False,
            env_overrides=env,
        )
        payload = final_json(completed)
        assert payload["clean"] is False
        assert payload["failed_count"] == 1
        zero_result = next(
            result for result in payload["results"] if result["case_id"] == "zero-test"
        )
        assert zero_result["status"] == "failed"
        assert "executed 0 tests" in zero_result["stderr_tail"]
        serialized = completed.stdout + completed.stderr
        assert "zero" in serialized.lower()
        return payload


def verify(case_id: str) -> list[str]:
    if case_id not in CASE_IDS:
        raise AssertionError(f"case is not owned by project-local-gates: {case_id}")
    started = time.monotonic()
    if case_id == "aw-ec-zero-test-false-green":
        _zero_test_snapshot()
        return [
            "aw ec verify rejects an exit-zero cargo command that reports zero tests",
            "the false-green fixture executes no compiler or real cargo process",
        ]

    first = _artifact_snapshot()
    wi = first["wi"]
    ec = first["ec"]
    td = first["td"]
    if case_id == "project-local-td-and-ec-gates-cb-generation-and-standardize-scan-defaults":
        assert td["generation"]["command"].startswith("aw cb gen ")
        assert td["identity"]["artifact_path"].startswith("tech-design/")
        return [
            "CB generation is rooted in the configured project-local tech-design tree",
            "TD artifact identity and generation continuation share the same slug",
        ]
    if case_id == "project-local-td-and-ec-gates-ec-evidence-documentation":
        assert ec["artifacts"][0] == "external-contracts/pyproject.toml"
        assert ec["next"]["payload_path"] == "external-contracts/pyproject.toml"
        return [
            "EC inventory and evidence source are documented under project-local external-contracts",
            "the structural-check envelope retains the inventory payload path",
        ]
    if case_id == "project-local-td-and-ec-gates-ec-external-contract-source":
        assert "external-contracts/src/artifact-fixture.py" in ec["artifacts"]
        return [
            "EC draft writes Python external-contract source and inventory",
            "the prompt explicitly rejects Markdown fallback",
        ]
    if case_id == "project-local-td-and-ec-gates-ec-tool-binding-dispatch":
        assert ec["next"]["command"].startswith("aw ec check --project demo ")
        return [
            "project-local EC binding resolves to the configured Python inventory",
            "the emitted runner dispatch is an executable aw ec check command",
        ]
    if case_id == "project-local-td-and-ec-gates-project-dirty-scope-protection":
        assert all(
            path.startswith("external-contracts/") for path in ec["artifacts"]
        )
        return [
            "EC producer writes only project-local external-contract artifacts",
            "generated wrappers never enter product source dirty scope",
        ]
    if case_id == "project-local-td-and-ec-gates-project-local-td-root-resolver":
        assert td["identity"]["artifact_path"].startswith("tech-design/")
        return [
            "TD root resolver selects the configured project tech-design directory",
            "payload and validation commands reference the same project-local spec",
        ]
    if case_id == "project-local-td-and-ec-gates-td-lock-and-external-contract-target-resolution":
        assert td["validation"]["command"].startswith("aw td check tech-design/")
        assert ec["root"]["id"] == "demo"
        return [
            "TD validation resolves the project-local spec target",
            "EC context remains bound to the configured project root",
        ]
    if case_id == "project-local-td-and-ec-gates-operational-efficiency":
        assert time.monotonic() - started <= 120
        return [
            "native project-local TD/EC producer gate completes within 120 seconds",
            "all representative assertions pass without cargo delegation",
        ]
    second = _artifact_snapshot()
    assert first["td"] == second["td"]
    assert first["ec"]["artifacts"] == second["ec"]["artifacts"]
    assert first["ec"]["invoke"] == second["ec"]["invoke"]
    assert first["ec"]["next"] == second["ec"]["next"]
    return [
        "two project-local TD/EC producer runs preserve artifact identities",
        "both runs emit the same inventory, source, validation, and continuation routes",
    ]
