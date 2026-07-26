"""Prove the canonical Python inventory and case sources agree exactly."""

from __future__ import annotations

import ast
import json
import tomllib
from pathlib import Path


EC_ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = Path(__file__).with_name("migration_reconciliation_manifest.json")
RUNNER_PREFIX = (
    "python3 apps/agentic-workflow/external-contracts/src/runner.py --case "
)


def _case_constants(path: Path) -> dict[str, object]:
    tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    values: dict[str, object] = {}
    for node in tree.body:
        if not isinstance(node, ast.Assign) or len(node.targets) != 1:
            continue
        target = node.targets[0]
        if isinstance(target, ast.Name):
            try:
                values[target.id] = ast.literal_eval(node.value)
            except ValueError:
                continue
    return values


def main() -> None:
    python_document = tomllib.loads(
        (EC_ROOT / "pyproject.toml").read_text(encoding="utf-8")
    )
    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    cases = python_document["tool"]["aw"]["python-ec"]["cases"]
    python_cases = {str(case["id"]): case for case in cases}
    assert len(python_cases) == len(cases), "duplicate Python EC case id"

    rust_invariant_ids = {
        str(case["id"])
        for cluster in manifest["clusters"]
        for case in cluster["cases"]
        if case["migration_status"] == "rust_invariant"
    }
    assert rust_invariant_ids.isdisjoint(python_cases)

    dimensions: dict[str, int] = {}
    for case_id, canonical in python_cases.items():
        dimension = str(canonical["dimension"])
        dimensions[dimension] = dimensions.get(dimension, 0) + 1
        assert dimension in {"behavior", "efficiency", "security", "stability"}
        assert canonical["target"] == "rust"
        assert canonical["command"] == f"{RUNNER_PREFIX}{case_id}"
        assert "cargo test" not in canonical["command"]
        assert not str(canonical["test_path"]).startswith(
            "apps/agentic-workflow/tests/"
        )

        source_path = EC_ROOT / str(canonical["test_path"])
        source = _case_constants(source_path)
        extended_constants = {
            "CAPABILITY_ID",
            "USE_CASE_ID",
            "DIMENSION",
            "TARGET_COMMAND",
            "ASSERTIONS",
        }
        assert source["CASE_ID"] == case_id
        projected_constants = extended_constants.intersection(source)
        assert projected_constants in (set(), extended_constants), (
            f"{case_id} has a partial inventory projection: "
            f"{sorted(projected_constants)}"
        )
        if projected_constants:
            assert source["CAPABILITY_ID"] == canonical["capability_id"]
            assert source["USE_CASE_ID"] == canonical["use_case_id"]
            assert source["DIMENSION"] == dimension
            assert source["TARGET_COMMAND"] == canonical["command"]
            assert tuple(source["ASSERTIONS"])
        assert any(
            isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
            and node.name == "verify"
            for node in ast.parse(
                source_path.read_text(encoding="utf-8"),
                filename=str(source_path),
            ).body
        )

        if "-operational-" in case_id:
            assert canonical["use_case_id"] == case_id, (
                f"{case_id} operational use_case_id must be case-local, got "
                f"{canonical['use_case_id']}"
            )
        if dimension in {"efficiency", "stability"}:
            assert canonical.get("threshold")

    assert dimensions.get("behavior", 0) > 0
    assert dimensions.get("efficiency", 0) > 0
    assert dimensions.get("stability", 0) > 0
    print(
        f"canonical Python inventory is self-consistent: {len(python_cases)} "
        f"cases, {len(rust_invariant_ids)} separately retained Rust invariants, "
        f"dimensions={json.dumps(dimensions, sort_keys=True)}"
    )


if __name__ == "__main__":
    main()
