"""Run one canonical AW external contract and emit digestable evidence."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import sys
import time
from pathlib import Path


EC_ROOT = Path(__file__).resolve().parents[1]
REPOSITORY_ROOT = Path(__file__).resolve().parents[4]


def _arguments() -> tuple[str, str, float, list[str]]:
    parser = argparse.ArgumentParser()
    parser.add_argument("--case", required=True)
    parser.add_argument(
        "--mode",
        choices=("behavior", "efficiency", "stability"),
        default="behavior",
    )
    parser.add_argument("--threshold-seconds", type=float, default=120.0)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    parsed = parser.parse_args()
    command = list(parsed.command)
    if command[:1] == ["--"]:
        command = command[1:]
    if not command:
        parser.error("the external oracle command is required after --")
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
