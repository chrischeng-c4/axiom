"""Run one canonical AW external contract and emit digestable evidence."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import re
import subprocess
import sys
import time
from pathlib import Path


EC_ROOT = Path(__file__).resolve().parents[1]
REPOSITORY_ROOT = Path(__file__).resolve().parents[4]


def _arguments() -> tuple[str, str | None, float, list[str]]:
    parser = argparse.ArgumentParser()
    parser.add_argument("--case", required=True)
    parser.add_argument(
        "--mode",
        choices=("behavior", "efficiency", "stability"),
        default=None,
    )
    parser.add_argument("--threshold-seconds", type=float, default=120.0)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    parsed = parser.parse_args()
    command = list(parsed.command)
    if command[:1] == ["--"]:
        command = command[1:]
    return parsed.case, parsed.mode, parsed.threshold_seconds, command


def _digest(value: str) -> str:
    return "sha256:" + hashlib.sha256(value.encode()).hexdigest()


def _run(command: list[str]) -> tuple[subprocess.CompletedProcess[str], float]:
    started = time.monotonic()
    completed = subprocess.run(
        command,
        cwd=REPOSITORY_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    return completed, time.monotonic() - started


def _run_case_implementation(
    case_id: str,
    requested_mode: str | None,
    threshold_seconds: float,
) -> int:
    source_path = EC_ROOT / "src" / "cases" / f"{case_id}.py"
    spec = importlib.util.spec_from_file_location(
        f"aw_external_contract_{case_id.replace('-', '_')}",
        source_path,
    )
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load Python EC implementation: {source_path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    verifier = getattr(module, "verify", None)
    if not callable(verifier):
        raise RuntimeError(f"Python EC implementation has no verify(): {source_path}")
    declared_mode = getattr(module, "DIMENSION", None)
    if declared_mode not in {"behavior", "efficiency", "security", "stability"}:
        raise RuntimeError(f"Python EC implementation has invalid DIMENSION: {source_path}")
    if requested_mode is not None and requested_mode != declared_mode:
        raise RuntimeError(
            f"requested mode {requested_mode} does not match declared "
            f"dimension {declared_mode}: {source_path}"
        )
    attempt_count = 2 if declared_mode == "stability" else 1
    attempts: list[dict[str, object]] = []
    assertion_sets: list[list[str]] = []
    for _ in range(attempt_count):
        started = time.monotonic()
        assertions = verifier()
        elapsed = time.monotonic() - started
        if (
            not isinstance(assertions, list)
            or not assertions
            or not all(
                isinstance(assertion, str) and assertion for assertion in assertions
            )
        ):
            raise RuntimeError(
                "Python EC verify() must return a non-empty list of assertions: "
                f"{source_path}"
            )
        if declared_mode == "efficiency" and elapsed > threshold_seconds:
            raise RuntimeError(
                f"Python EC efficiency threshold exceeded: {elapsed:.3f}s > "
                f"{threshold_seconds:.3f}s"
            )
        assertion_sets.append(assertions)
        attempts.append(
            {
                "elapsed_ms": round(elapsed * 1000),
                "exit_code": 0,
                "assertion_count": len(assertions),
                "assertions_digest": _digest(
                    json.dumps(assertions, ensure_ascii=True, separators=(",", ":"))
                ),
            }
        )
    if declared_mode == "stability" and assertion_sets[0] != assertion_sets[1]:
        raise RuntimeError(
            f"Python EC stability attempts produced different assertions: {source_path}"
        )
    evidence = {
        "protocol": "aw.python-ec.evidence.v1",
        "case_id": case_id,
        "mode": declared_mode,
        "implementation": str(source_path.relative_to(REPOSITORY_ROOT)),
        "exit_code": 0,
        "threshold_seconds": threshold_seconds
        if declared_mode == "efficiency"
        else None,
        "assertions": assertion_sets[-1],
        "attempts": attempts,
    }
    evidence_path = EC_ROOT / "evidence" / f"{case_id}.json"
    evidence_path.parent.mkdir(parents=True, exist_ok=True)
    evidence_path.write_text(
        json.dumps(evidence, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(json.dumps(evidence, sort_keys=True))
    return 0


def _test_summary(output: str) -> tuple[int, int]:
    results = re.findall(
        r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed;",
        output,
    )
    return (
        sum(int(passed) for passed, _ in results),
        sum(int(failed) for _, failed in results),
    )


def main() -> int:
    case_id, mode, threshold_seconds, command = _arguments()
    source_path = EC_ROOT / "src" / "cases" / f"{case_id}.py"
    if not source_path.is_file():
        raise SystemExit(f"unknown EC case: {case_id}")
    if not command:
        return _run_case_implementation(case_id, mode, threshold_seconds)

    mode = mode or "behavior"
    attempt_count = 2 if mode == "stability" else 1
    attempts = [_run(command) for _ in range(attempt_count)]
    for completed, _ in attempts:
        sys.stdout.write(completed.stdout)
        sys.stderr.write(completed.stderr)

    summaries = [
        _test_summary(completed.stdout + completed.stderr)
        for completed, _ in attempts
    ]
    exit_code = next(
        (
            completed.returncode
            for completed, _ in attempts
            if completed.returncode != 0
        ),
        0,
    )
    if mode != "behavior":
        if any(passed == 0 or failed != 0 for passed, failed in summaries):
            exit_code = exit_code or 1
        if mode == "efficiency" and attempts[0][1] > threshold_seconds:
            exit_code = exit_code or 1
        if mode == "stability" and len(set(summaries)) != 1:
            exit_code = exit_code or 1

    evidence = {
        "protocol": "aw.python-ec.evidence.v1",
        "case_id": case_id,
        "mode": mode,
        "command": command,
        "exit_code": exit_code,
        "threshold_seconds": threshold_seconds,
        "attempts": [
            {
                "elapsed_ms": round(elapsed * 1000),
                "exit_code": completed.returncode,
                "passed_tests": summary[0],
                "failed_tests": summary[1],
                "stdout_digest": _digest(completed.stdout),
                "stderr_digest": _digest(completed.stderr),
            }
            for (completed, elapsed), summary in zip(attempts, summaries)
        ],
    }
    evidence_path = EC_ROOT / "evidence" / f"{case_id}.json"
    evidence_path.parent.mkdir(parents=True, exist_ok=True)
    evidence_path.write_text(
        json.dumps(evidence, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
