"""Inventory and enforce the Rust-wrapper to native-Python EC migration."""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
import sys
import tomllib
from pathlib import Path
from typing import Any


EC_ROOT = Path(__file__).resolve().parents[1]
PROJECT_ROOT = EC_ROOT.parent
REPOSITORY_ROOT = PROJECT_ROOT.parents[1]
MANIFEST_PATH = Path(__file__).with_name("migration_reconciliation_manifest.json")
PYPROJECT_PATH = EC_ROOT / "pyproject.toml"
CASES_ROOT = Path(__file__).with_name("cases")
LEGACY_TEST_ROOT = PROJECT_ROOT / "tests"


def _arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    subcommands = parser.add_subparsers(dest="command", required=True)
    verify = subcommands.add_parser("verify")
    selection = verify.add_mutually_exclusive_group(required=True)
    selection.add_argument("--baseline", action="store_true")
    selection.add_argument("--policy", action="store_true")
    selection.add_argument("--cluster")
    return parser.parse_args()


def _load_manifest() -> dict[str, Any]:
    return json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))


def _manifest_digest(manifest: dict[str, Any]) -> str:
    canonical = json.dumps(manifest, sort_keys=True, separators=(",", ":"))
    return "sha256:" + hashlib.sha256(canonical.encode()).hexdigest()


def _configured_cases() -> dict[str, dict[str, Any]]:
    document = tomllib.loads(PYPROJECT_PATH.read_text(encoding="utf-8"))
    cases = document["tool"]["aw"]["python-ec"]["cases"]
    return {case["id"]: case for case in cases}


def _legacy_files() -> set[str]:
    return {
        str(path.relative_to(REPOSITORY_ROOT))
        for path in LEGACY_TEST_ROOT.rglob("*.rs")
    }


def _has_verify(case_id: str) -> bool:
    source = CASES_ROOT / f"{case_id}.py"
    if not source.is_file():
        return False
    tree = ast.parse(source.read_text(encoding="utf-8"), filename=str(source))
    return any(
        isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
        and node.name == "verify"
        for node in tree.body
    )


def _delegated_entries(manifest: dict[str, Any]) -> list[dict[str, Any]]:
    return [
        {
            **case,
            "cluster": cluster["id"],
            "owner_wi": cluster["owner_wi"],
        }
        for cluster in manifest["clusters"]
        for case in cluster["cases"]
    ]


def _duplicates(values: list[str]) -> int:
    return len(values) - len(set(values))


def _baseline(manifest: dict[str, Any]) -> dict[str, Any]:
    configured = _configured_cases()
    entries = _delegated_entries(manifest)
    expected_legacy = manifest["legacy_rust_files"]
    expected_cases = [entry["id"] for entry in entries]
    actual_legacy = _legacy_files()
    actual_delegated = {
        case_id
        for case_id, case in configured.items()
        if "cargo test" in case.get("command", "")
    }
    duplicate = _duplicates(expected_legacy) + _duplicates(expected_cases)
    unmatched = len(actual_legacy.symmetric_difference(expected_legacy))
    unmatched += len(actual_delegated.symmetric_difference(expected_cases))
    result = {
        "schema": manifest["schema"],
        "legacy_files": len(expected_legacy),
        "delegated_cases": len(expected_cases),
        "unmatched": unmatched,
        "duplicate": duplicate,
        "manifest_digest": _manifest_digest(manifest),
    }
    if (
        result["legacy_files"] != 164
        or result["delegated_cases"] != 112
        or unmatched
        or duplicate
    ):
        raise RuntimeError(json.dumps(result, sort_keys=True))
    return result


def _policy(manifest: dict[str, Any]) -> dict[str, Any]:
    configured = _configured_cases()
    entries = _delegated_entries(manifest)
    expected_legacy = set(manifest["legacy_rust_files"])
    expected_cases = {entry["id"] for entry in entries}
    unexpected_rust = sorted(_legacy_files() - expected_legacy)
    current_cargo = {
        case_id
        for case_id, case in configured.items()
        if "cargo test" in case.get("command", "")
    }
    unexpected_cargo = sorted(current_cargo - expected_cases)
    regressed_native = sorted(
        entry["id"]
        for entry in entries
        if entry["migration_status"] == "native"
        and (
            entry["id"] in current_cargo
            or not _has_verify(entry["id"])
        )
    )
    if unexpected_rust or unexpected_cargo or regressed_native:
        raise RuntimeError(
            json.dumps(
                {
                    "unexpected_rust_wrappers": unexpected_rust,
                    "unexpected_cargo_oracles": unexpected_cargo,
                    "regressed_native_cases": regressed_native,
                },
                sort_keys=True,
            )
        )
    return {
        "schema": manifest["schema"],
        "policy": "pass",
        "legacy_ceiling": len(expected_legacy),
        "delegated_ceiling": len(expected_cases),
        "current_legacy": len(_legacy_files()),
        "current_delegated": len(current_cargo),
        "manifest_digest": _manifest_digest(manifest),
    }


def _cluster(manifest: dict[str, Any], cluster_id: str) -> dict[str, Any]:
    configured = _configured_cases()
    cluster = next(
        (item for item in manifest["clusters"] if item["id"] == cluster_id),
        None,
    )
    if cluster is None:
        raise RuntimeError(f"unknown migration cluster: {cluster_id}")
    failures: list[str] = []
    for entry in cluster["cases"]:
        case_id = entry["id"]
        configured_case = configured.get(case_id)
        if configured_case is None:
            failures.append(f"{case_id}: missing pyproject inventory")
            continue
        if "cargo test" in configured_case.get("command", ""):
            failures.append(f"{case_id}: still delegates to cargo test")
        if not _has_verify(case_id):
            failures.append(f"{case_id}: native verify() is missing")
        if entry["migration_status"] != "native":
            failures.append(f"{case_id}: migration status is not native")
        disposition = entry["rust_disposition"]
        legacy_path_value = entry["legacy_test_path"]
        legacy_path = (
            REPOSITORY_ROOT / legacy_path_value
            if legacy_path_value is not None
            else None
        )
        if disposition == "pending":
            failures.append(f"{case_id}: Rust disposition is pending")
        elif disposition == "removed":
            if legacy_path is None or legacy_path.exists():
                failures.append(f"{case_id}: removed Rust path still exists")
        elif disposition.startswith("relocated:"):
            relocated = REPOSITORY_ROOT / disposition.removeprefix("relocated:")
            if legacy_path is None or legacy_path.exists() or not relocated.is_file():
                failures.append(f"{case_id}: relocation evidence is incomplete")
        elif disposition == "retained_colocated":
            if legacy_path is not None:
                failures.append(f"{case_id}: retained coverage is still app-level")
        else:
            failures.append(f"{case_id}: unsupported disposition {disposition}")
    if failures:
        raise RuntimeError("\n".join(failures))
    return {
        "schema": manifest["schema"],
        "cluster": cluster_id,
        "owner_wi": cluster["owner_wi"],
        "case_count": len(cluster["cases"]),
        "status": "native",
        "manifest_digest": _manifest_digest(manifest),
    }


def main() -> int:
    args = _arguments()
    manifest = _load_manifest()
    if args.baseline:
        result = _baseline(manifest)
    elif args.policy:
        result = _policy(manifest)
    else:
        result = _cluster(manifest, args.cluster)
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (KeyError, OSError, RuntimeError, tomllib.TOMLDecodeError) as error:
        print(f"migration reconciliation failed: {error}", file=sys.stderr)
        raise SystemExit(1) from error
