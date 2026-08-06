"""Execute the independent openapi-codegen external-contract runner according to aw.python-artifact protocol."""

from __future__ import annotations

import importlib.util
import json
import os
import sys
from pathlib import Path
from typing import Any

EC_ROOT = Path(__file__).resolve().parent.parent
REPO_ROOT = EC_ROOT.parents[2]
SRC_DIR = EC_ROOT / "src"
TECH_DESIGN_SRC = REPO_ROOT / "libs" / "openapi-codegen" / "tech-design" / "src"

DECLARED_CASES: dict[str, int] = {
    "tolerant-openapi-document-subset-behavior": 16,
    "tolerant-openapi-document-subset-security": 14,
    "deterministic-identifier-naming-behavior": 14,
    "deterministic-identifier-naming-security": 14,
    "language-neutral-operation-ir-behavior": 18,
    "language-neutral-operation-ir-security": 16,
    "per-language-type-mapping-behavior": 18,
    "per-language-type-mapping-security": 16,
    "versioned-target-profiles-behavior": 16,
    "versioned-target-profiles-security": 16,
    "contained-output-materialization-behavior": 14,
    "contained-output-materialization-security": 16,
}

REQUIRED_ARTIFACT_PROTOCOL = "aw.python-artifact.v1"
RESULT_SCHEMA_VERSION = "aw.python-artifact.result.v1"


def print_envelope(
    status: str,
    source_digest: str,
    lock_digest: str,
    evidence_paths: list[str],
) -> None:
    envelope = {
        "schema_version": RESULT_SCHEMA_VERSION,
        "status": status,
        "source_digest": source_digest,
        "dependency_lock_digest": lock_digest,
        "evidence": evidence_paths,
    }
    print(json.dumps(envelope, indent=2, sort_keys=True))


def strict_equals(a: Any, b: Any) -> bool:
    """Type-aware deep equality comparison rejecting cross-type Python equality."""
    if type(a) is not type(b):
        return False
    if isinstance(a, (int, float, str, bool, bytes, type(None))):
        return a == b
    if isinstance(a, (list, tuple)):
        if len(a) != len(b):
            return False
        return all(strict_equals(x, y) for x, y in zip(a, b))
    if isinstance(a, (set, frozenset)):
        if len(a) != len(b):
            return False
        b_items = list(b)
        matched_indices: set[int] = set()
        for item_a in a:
            found_pair = False
            for idx, item_b in enumerate(b_items):
                if idx not in matched_indices and strict_equals(item_a, item_b):
                    matched_indices.add(idx)
                    found_pair = True
                    break
            if not found_pair:
                return False
        return True
    if isinstance(a, dict):
        if len(a) != len(b):
            return False
        b_items = list(b.items())
        matched_indices: set[int] = set()
        for k_a, v_a in a.items():
            found_pair = False
            for idx, (k_b, v_b) in enumerate(b_items):
                if idx not in matched_indices and strict_equals(k_a, k_b) and strict_equals(v_a, v_b):
                    matched_indices.add(idx)
                    found_pair = True
                    break
            if not found_pair:
                return False
        return True
    try:
        return a == b
    except Exception:
        return False


def validate_result(
    case_id: str,
    result: Any,
    min_checks: int,
    source_digest: str,
    lock_digest: str,
) -> tuple[bool, str, dict[str, Any] | None]:
    if type(result) is not dict:
        return False, f"Verifier result for {case_id} must be a dict, got {type(result).__name__}", None

    if "case_id" not in result or result["case_id"] != case_id:
        return False, f"Case ID mismatch: expected {case_id!r}, got {result.get('case_id')!r}", None

    min_checks_val = result.get("minimum_checks")
    if type(min_checks_val) is not int or min_checks_val != min_checks:
        return False, f"Minimum checks mismatch: expected integer {min_checks}, got {min_checks_val!r}", None

    passed_val = result.get("passed")
    if type(passed_val) is not bool or passed_val is not True:
        return False, f"Case passed status is not boolean True: got {passed_val!r}", None

    checks = result.get("checks")
    if type(checks) is not list:
        return False, f"Checks must be a list, got {type(checks).__name__}", None

    if len(checks) != min_checks:
        return False, f"Check count {len(checks)} does not match exact required floor of {min_checks} for {case_id}", None

    seen_names: set[str] = set()
    for idx, check in enumerate(checks):
        if type(check) is not dict:
            return False, f"Check at index {idx} must be a dict", None

        name = check.get("name")
        if type(name) is not str or not name.strip():
            return False, f"Check at index {idx} missing valid non-empty 'name' string", None

        if name in seen_names:
            return False, f"Duplicate check name {name!r} in verifier result", None
        seen_names.add(name)

        if "observed" not in check or "expected" not in check:
            return False, f"Check {name!r} missing required 'observed' or 'expected' field", None

        chk_passed = check.get("passed")
        if type(chk_passed) is not bool or chk_passed is not True:
            return False, f"Check {name!r} passed status is not boolean True: got {chk_passed!r}", None

        obs_val = check["observed"]
        exp_val = check["expected"]
        if not strict_equals(obs_val, exp_val):
            return (
                False,
                f"Check {name!r} observed value {obs_val!r} (type {type(obs_val).__name__}) does not strictly equal expected value {exp_val!r} (type {type(exp_val).__name__})",
                None,
            )

    evidence: dict[str, Any] = {
        "case_id": case_id,
        "minimum_checks": min_checks,
        "passed": True,
        "checks": checks,
        "source_digest": source_digest,
        "lock_digest": lock_digest,
    }
    return True, "", evidence


def run_case(case_id: str, src_dir: Path | None = None) -> tuple[int, list[str], str]:
    artifact_proto = os.environ.get("AW_PYTHON_ARTIFACT_PROTOCOL", "")
    source_digest = os.environ.get("AW_PYTHON_ARTIFACT_SOURCE_DIGEST", "")
    lock_digest = os.environ.get("AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST", "")
    evidence_dir_str = os.environ.get("AW_PYTHON_ARTIFACT_EVIDENCE_DIR", "")

    if artifact_proto != REQUIRED_ARTIFACT_PROTOCOL:
        return (
            1,
            [],
            f"Invalid AW_PYTHON_ARTIFACT_PROTOCOL: expected {REQUIRED_ARTIFACT_PROTOCOL!r}, got {artifact_proto!r}",
        )

    if not source_digest:
        return 1, [], "AW_PYTHON_ARTIFACT_SOURCE_DIGEST environment variable is missing or empty"

    if not lock_digest:
        return 1, [], "AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST environment variable is missing or empty"

    if not evidence_dir_str:
        return 1, [], "AW_PYTHON_ARTIFACT_EVIDENCE_DIR environment variable is missing or empty"

    if case_id not in DECLARED_CASES:
        return 2, [], f"Undeclared command: {case_id!r}. Runner dispatches only declared cases."

    min_checks = DECLARED_CASES[case_id]
    target_src = src_dir or SRC_DIR

    if TECH_DESIGN_SRC.is_dir() and str(TECH_DESIGN_SRC) not in sys.path:
        sys.path.insert(0, str(TECH_DESIGN_SRC))

    module_name = case_id.replace("-", "_")
    module_path = target_src / f"{case_id}.py"

    if not module_path.is_file():
        return 1, [], f"Case module file does not exist: {module_path}"

    try:
        spec = importlib.util.spec_from_file_location(module_name, module_path)
        if spec is None or spec.loader is None:
            return 1, [], f"Failed to create spec for module at {module_path}"
        mod = importlib.util.module_from_spec(spec)
        sys.modules[module_name] = mod
        spec.loader.exec_module(mod)
    except Exception as exc:
        return 1, [], f"Failed to import module {case_id}: {exc}"

    entrypoint_name = f"verify_{module_name}"
    entrypoint = getattr(mod, entrypoint_name, None)
    if entrypoint is None or not callable(entrypoint):
        return 1, [], f"Entrypoint {entrypoint_name}() missing or non-callable in {case_id}"

    try:
        result = entrypoint()
    except Exception as exc:
        return 1, [], f"Verifier exception in {entrypoint_name}(): {exc}"

    valid, err_msg, evidence = validate_result(case_id, result, min_checks, source_digest, lock_digest)
    if not valid or evidence is None:
        return 1, [], err_msg

    evidence_dir = Path(evidence_dir_str)
    try:
        evidence_dir.mkdir(parents=True, exist_ok=True)
        evidence_file = evidence_dir / f"{case_id}.json"
        with open(evidence_file, "w", encoding="utf-8") as f:
            json.dump(evidence, f, indent=2, sort_keys=True)
            f.write("\n")
    except Exception as exc:
        return 1, [], f"Failed to write evidence file to {evidence_dir_str}: {exc}"

    relative_ev_path = f"evidence/{case_id}.json"
    return 0, [relative_ev_path], ""


def main(argv: list[str]) -> int:
    source_digest = os.environ.get("AW_PYTHON_ARTIFACT_SOURCE_DIGEST", "")
    lock_digest = os.environ.get("AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST", "")

    args = argv[1:]
    if not args:
        print_envelope("failed", source_digest, lock_digest, [])
        return 2

    if args[0] == "--case":
        if len(args) != 2:
            print_envelope("failed", source_digest, lock_digest, [])
            return 2
        case_id = args[1]
    else:
        if len(args) != 1:
            print_envelope("failed", source_digest, lock_digest, [])
            return 2
        case_id = args[0]

    code, ev_paths, msg = run_case(case_id)
    if code != 0:
        print_envelope("failed", source_digest, lock_digest, [])
        return code

    print_envelope("passed", source_digest, lock_digest, ev_paths)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
