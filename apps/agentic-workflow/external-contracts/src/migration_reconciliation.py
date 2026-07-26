"""Inventory and enforce the Rust-wrapper to native-Python EC migration."""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
import subprocess
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
GUIDANCE_PATHS = (
    REPOSITORY_ROOT / "CONTRIBUTING.md",
    REPOSITORY_ROOT / "AGENTS.md",
    PROJECT_ROOT / "README.md",
    PROJECT_ROOT / "CAPABILITIES.md",
    REPOSITORY_ROOT
    / "apps/agentic-workflow/templates/cli/mainthread/agents/aw-dev.md",
    REPOSITORY_ROOT
    / "apps/agentic-workflow/templates/cli/mainthread/agents/aw-ec-writer.md",
    REPOSITORY_ROOT
    / "apps/agentic-workflow/templates/cli/mainthread/agents/aw-ec-reviewer.md",
    REPOSITORY_ROOT / ".agents/agents/aw-dev.md",
    REPOSITORY_ROOT / ".agents/agents/aw-ec-writer.md",
    REPOSITORY_ROOT / ".agents/agents/aw-ec-reviewer.md",
)


def _arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    subcommands = parser.add_subparsers(dest="command", required=True)
    verify = subcommands.add_parser("verify")
    selection = verify.add_mutually_exclusive_group(required=True)
    selection.add_argument("--baseline", action="store_true")
    selection.add_argument("--policy", action="store_true")
    selection.add_argument("--cluster")
    selection.add_argument("--projections", action="store_true")
    selection.add_argument("--guidance", action="store_true")
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


def _declared_target_delegates_to_cargo(case_id: str) -> bool:
    source = CASES_ROOT / f"{case_id}.py"
    if not source.is_file():
        return False
    tree = ast.parse(source.read_text(encoding="utf-8"), filename=str(source))
    for node in tree.body:
        if not isinstance(node, ast.Assign):
            continue
        if not any(
            isinstance(target, ast.Name) and target.id == "TARGET_COMMAND"
            for target in node.targets
        ):
            continue
        value = ast.literal_eval(node.value)
        return isinstance(value, str) and "cargo test" in value
    return False


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
    entries = _delegated_entries(manifest)
    expected_legacy = manifest["legacy_rust_files"]
    expected_cases = [entry["id"] for entry in entries]
    duplicate = _duplicates(expected_legacy) + _duplicates(expected_cases)
    malformed = sum(
        1
        for path in expected_legacy
        if not path.startswith("apps/agentic-workflow/tests/")
        or not path.endswith(".rs")
    )
    malformed += sum(
        1
        for entry in entries
        if not entry["id"]
        or not entry["cluster"]
        or not entry["owner_wi"]
        or (
            entry["legacy_test_path"] is not None
            and entry["legacy_test_path"] not in expected_legacy
        )
    )
    result = {
        "schema": manifest["schema"],
        "legacy_files": len(expected_legacy),
        "delegated_cases": len(expected_cases),
        "operational_cases_without_legacy_path": sum(
            1 for entry in entries if entry["legacy_test_path"] is None
        ),
        "unmatched": malformed,
        "duplicate": duplicate,
        "manifest_digest": _manifest_digest(manifest),
        "projection": "frozen_pre_migration_baseline",
    }
    if (
        result["legacy_files"] != 164
        or result["delegated_cases"] != 112
        or result["operational_cases_without_legacy_path"] != 2
        or malformed
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
            or _declared_target_delegates_to_cargo(entry["id"])
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
        migration_status = entry["migration_status"]
        disposition = entry["rust_disposition"]
        legacy_path_value = entry["legacy_test_path"]
        legacy_path = (
            REPOSITORY_ROOT / legacy_path_value
            if legacy_path_value is not None
            else None
        )
        if migration_status == "rust_invariant":
            if configured_case is not None:
                failures.append(
                    f"{case_id}: Rust invariant must not remain in Python EC inventory"
                )
            if not disposition.startswith("relocated:"):
                failures.append(
                    f"{case_id}: Rust invariant must name its relocated owner"
                )
                continue
            relocated = REPOSITORY_ROOT / disposition.removeprefix("relocated:")
            if legacy_path is None or legacy_path.exists() or not relocated.is_file():
                failures.append(f"{case_id}: Rust invariant relocation is incomplete")
            continue
        if configured_case is None:
            failures.append(f"{case_id}: missing pyproject inventory")
            continue
        if "cargo test" in configured_case.get("command", ""):
            failures.append(f"{case_id}: still delegates to cargo test")
        if not _has_verify(case_id):
            failures.append(f"{case_id}: native verify() is missing")
        if _declared_target_delegates_to_cargo(case_id):
            failures.append(f"{case_id}: TARGET_COMMAND still delegates to cargo test")
        if migration_status != "native":
            failures.append(f"{case_id}: migration status is not native")
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
        "native_python_ec_count": sum(
            1 for entry in cluster["cases"] if entry["migration_status"] == "native"
        ),
        "rust_invariant_count": sum(
            1
            for entry in cluster["cases"]
            if entry["migration_status"] == "rust_invariant"
        ),
        "status": "reconciled",
        "manifest_digest": _manifest_digest(manifest),
    }


def _assigned_string_tuple(path: Path, name: str) -> tuple[str, ...]:
    tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    for node in tree.body:
        if not isinstance(node, ast.Assign):
            continue
        if not any(
            isinstance(target, ast.Name) and target.id == name
            for target in node.targets
        ):
            continue
        value = ast.literal_eval(node.value)
        if isinstance(value, tuple) and all(isinstance(item, str) for item in value):
            return value
    return ()


def _run_projection_check(command: list[str]) -> dict[str, Any]:
    completed = subprocess.run(
        command,
        cwd=REPOSITORY_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(
            f"projection check failed: {' '.join(command)}\n"
            f"stdout={completed.stdout}\nstderr={completed.stderr}"
        )
    return json.loads(completed.stdout)


def _lock_projection() -> dict[str, Any]:
    command = [
        str(REPOSITORY_ROOT / "target/debug/aw"),
        "ec",
        "lock",
        "--project",
        "agentic-workflow",
        "--check",
        "--json",
    ]
    completed = subprocess.run(
        command,
        cwd=REPOSITORY_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    try:
        result = json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise RuntimeError(
            f"EC lock check did not emit JSON\nstdout={completed.stdout}\n"
            f"stderr={completed.stderr}"
        ) from error
    if completed.returncode == 0 and result.get("clean") is True:
        return result
    if result.get("status") == "migration_required":
        # Lock removal is deliberately reviewed before `aw ec lock --wi` may
        # replace it.  Returning success here breaks the review/lock cycle;
        # the same gate reports clean only after accepted evidence updates it.
        return result
    raise RuntimeError(
        f"EC lock projection is neither clean nor review-ready\n"
        f"stdout={completed.stdout}\nstderr={completed.stderr}"
    )


def _projections(manifest: dict[str, Any]) -> dict[str, Any]:
    configured = _configured_cases()
    entries = _delegated_entries(manifest)
    migration_ids = {
        entry["id"] for entry in entries if entry["migration_status"] == "native"
    }
    rust_invariant_ids = {
        entry["id"]
        for entry in entries
        if entry["migration_status"] == "rust_invariant"
    }
    cargo_commands = sorted(
        case_id
        for case_id, case in configured.items()
        if "cargo test" in str(case.get("command", ""))
    )
    legacy_test_paths = sorted(
        case_id
        for case_id, case in configured.items()
        if str(case.get("test_path", "")).startswith(
            "apps/agentic-workflow/tests/"
        )
        or str(case.get("test_path", "")).endswith(".rs")
    )
    indirect_cargo = sorted(
        path.name
        for path in CASES_ROOT.glob("*.py")
        if any("cargo test" in command for command in _assigned_string_tuple(path, "TARGET_COMMANDS"))
    )
    unresolved_dispositions = sorted(
        entry["id"]
        for entry in entries
        if entry["migration_status"] not in {"native", "rust_invariant"}
        or entry["rust_disposition"] == "pending"
    )
    projected_rust_invariants = sorted(rust_invariant_ids.intersection(configured))
    wrong_targets = sorted(
        case_id
        for case_id in migration_ids
        if configured.get(case_id, {}).get("target") != "rust"
    )
    project_inventory = (PROJECT_ROOT / "aw.toml").read_text(encoding="utf-8")
    legacy_project_projection = (
        "AW-EC-BEGIN" in project_inventory or "[aw.ec.generated]" in project_inventory
    )
    reconciliation = _run_projection_check(
        [
            sys.executable,
            str(Path(__file__).with_name("claim_reconciliation.py")),
        ]
    )
    lock = _lock_projection()
    lock_status = (
        "clean" if lock.get("clean") is True else "review_ready_migration_required"
    )
    failures = {
        "cargo_delegating_commands": cargo_commands,
        "legacy_test_paths": legacy_test_paths,
        "indirect_cargo_delegation": indirect_cargo,
        "unresolved_migration_cases": unresolved_dispositions,
        "projected_rust_invariants": projected_rust_invariants,
        "wrong_product_targets": wrong_targets,
        "legacy_project_projection": legacy_project_projection,
        "claim_reconciliation_status": reconciliation.get("status"),
        "ec_lock_status": lock_status,
    }
    if (
        cargo_commands
        or legacy_test_paths
        or indirect_cargo
        or unresolved_dispositions
        or projected_rust_invariants
        or wrong_targets
        or legacy_project_projection
        or reconciliation.get("status") != "clean"
    ):
        raise RuntimeError(json.dumps(failures, sort_keys=True))
    return {
        "schema": manifest["schema"],
        "status": "clean",
        "case_count": len(configured),
        "native_migration_cases": len(migration_ids),
        "rust_invariant_cases": len(rust_invariant_ids),
        "cargo_delegating_commands": 0,
        "legacy_test_paths": 0,
        "indirect_cargo_delegation": 0,
        "claim_reconciliation_status": "clean",
        "ec_lock_status": lock_status,
        "manifest_digest": _manifest_digest(manifest),
    }


def _guidance(manifest: dict[str, Any]) -> dict[str, Any]:
    missing_paths = sorted(
        str(path.relative_to(REPOSITORY_ROOT))
        for path in GUIDANCE_PATHS
        if not path.is_file()
    )
    texts = {
        str(path.relative_to(REPOSITORY_ROOT)): path.read_text(encoding="utf-8")
        for path in GUIDANCE_PATHS
        if path.is_file()
    }
    forbidden = (
        "apps/agentic-workflow/tests/",
        "tests/cli/tests/",
        "generated EC test",
        "EC inventory in aw.toml",
    )
    stale_references = sorted(
        f"{path}:{token}"
        for path, text in texts.items()
        for token in forbidden
        if token in text
    )
    required_fragments = {
        "CONTRIBUTING.md": (
            "apps/agentic-workflow/external-contracts/",
            "colocated Rust invariants",
            "src/**",
        ),
        "AGENTS.md": (
            "apps/agentic-workflow/external-contracts/",
            "colocated Rust invariants",
            "src/**",
        ),
        "apps/agentic-workflow/README.md": (
            "external-contracts/",
            "src/**",
            "there is no app-level Rust EC wrapper tree",
        ),
        "apps/agentic-workflow/CAPABILITIES.md": (
            "canonical 44-case inventory",
            "37 migrated Python cases",
            "75 separately retained Rust invariants",
        ),
    }
    missing_fragments = sorted(
        f"{path}:{fragment}"
        for path, fragments in required_fragments.items()
        for fragment in fragments
        if fragment not in texts.get(path, "")
    )
    if missing_paths or stale_references or missing_fragments:
        raise RuntimeError(
            json.dumps(
                {
                    "missing_paths": missing_paths,
                    "stale_references": stale_references,
                    "missing_fragments": missing_fragments,
                },
                sort_keys=True,
            )
        )
    return {
        "schema": manifest["schema"],
        "status": "clean",
        "checked_paths": len(texts),
        "legacy_app_test_references": 0,
        "python_ec_guidance": "external-contracts/pyproject.toml + src/cases/*.py",
        "rust_invariant_guidance": "semantic src/** owners",
        "manifest_digest": _manifest_digest(manifest),
    }


def main() -> int:
    args = _arguments()
    manifest = _load_manifest()
    if args.baseline:
        result = _baseline(manifest)
    elif args.policy:
        result = _policy(manifest)
    elif args.projections:
        result = _projections(manifest)
    elif args.guidance:
        result = _guidance(manifest)
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
