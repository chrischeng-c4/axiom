"""Run one Guard Python external contract and emit digestable evidence."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import sys
import time
from pathlib import Path


EC_ROOT = Path(__file__).resolve().parents[1]
REPOSITORY_ROOT = Path(__file__).resolve().parents[4]
SRC_ROOT = EC_ROOT / "src"
INVENTORY_PATH = EC_ROOT / "pyproject.toml"


def _digest(value: object) -> str:
    encoded = json.dumps(value, sort_keys=True, separators=(",", ":"))
    return "sha256:" + hashlib.sha256(encoded.encode()).hexdigest()


def _declared_test_paths() -> dict[str, Path]:
    """Read the exact id/test_path pairs from the Python EC inventory."""
    paths: dict[str, Path] = {}
    current: dict[str, str] | None = None
    for raw_line in INVENTORY_PATH.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if line == "[[tool.aw.python-ec.cases]]":
            if current is not None and {"id", "test_path"} <= current.keys():
                paths[current["id"]] = EC_ROOT / current["test_path"]
            current = {}
            continue
        if line.startswith("[") and current is not None:
            if {"id", "test_path"} <= current.keys():
                paths[current["id"]] = EC_ROOT / current["test_path"]
            current = None
            continue
        if current is None or "=" not in line:
            continue
        key, raw_value = (part.strip() for part in line.split("=", 1))
        if key in {"id", "test_path"}:
            value = json.loads(raw_value)
            if not isinstance(value, str):
                raise RuntimeError(f"Guard EC inventory {key} must be a string")
            current[key] = value
    if current is not None and {"id", "test_path"} <= current.keys():
        paths[current["id"]] = EC_ROOT / current["test_path"]
    return paths


def _declared_source_path(case_id: str) -> Path:
    source_path = _declared_test_paths().get(case_id)
    if source_path is None or not source_path.is_file():
        raise SystemExit(f"unknown Guard EC case: {case_id}")
    resolved = source_path.resolve()
    try:
        resolved.relative_to(SRC_ROOT.resolve())
    except ValueError as error:
        raise RuntimeError(
            f"Guard EC test_path escapes src/: {source_path}"
        ) from error
    if resolved.suffix != ".py":
        raise RuntimeError(f"Guard EC test_path must be Python: {source_path}")
    return resolved


def _load_verifier(case_id: str):
    source_path = _declared_source_path(case_id)
    sys.path.insert(0, str(SRC_ROOT))
    spec = importlib.util.spec_from_file_location(
        f"guard_external_contract_{case_id.replace('-', '_')}",
        source_path,
    )
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load Python EC implementation: {source_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    verifier = getattr(module, "verify", None)
    if not callable(verifier):
        raise RuntimeError(f"Python EC implementation has no verify(): {source_path}")
    dimension = getattr(module, "DIMENSION", None)
    if dimension not in {"behavior", "security", "stability"}:
        raise RuntimeError(
            f"Guard EC implementation has invalid DIMENSION: {source_path}"
        )
    return source_path, verifier, dimension


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--case", required=True)
    parsed = parser.parse_args()

    source_path, verifier, dimension = _load_verifier(parsed.case)
    started = time.monotonic()
    assertions = verifier()
    elapsed_ms = round((time.monotonic() - started) * 1000)
    if (
        not isinstance(assertions, list)
        or not assertions
        or not all(isinstance(assertion, str) and assertion for assertion in assertions)
    ):
        raise RuntimeError("Guard EC verify() must return non-empty assertion strings")

    evidence = {
        "protocol": "aw.python-ec.evidence.v1",
        "case_id": parsed.case,
        "mode": dimension,
        "implementation": str(source_path.relative_to(REPOSITORY_ROOT)),
        "exit_code": 0,
        "assertions": assertions,
        "attempts": [
            {
                "elapsed_ms": elapsed_ms,
                "exit_code": 0,
                "assertion_count": len(assertions),
                "assertions_digest": _digest(assertions),
            }
        ],
    }
    evidence_path = EC_ROOT / "evidence" / f"{parsed.case}.json"
    evidence_path.parent.mkdir(parents=True, exist_ok=True)
    evidence_path.write_text(
        json.dumps(evidence, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(json.dumps(evidence, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
