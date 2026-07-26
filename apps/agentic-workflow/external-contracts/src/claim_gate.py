"""Independent executor for one generated capability-claim EC declaration."""

from __future__ import annotations

import argparse
import ast
import json
import re
import shlex
import subprocess
from pathlib import Path


EC_ROOT = Path(__file__).resolve().parents[1]
REPOSITORY_ROOT = Path(__file__).resolve().parents[4]
CASE_ROOT = EC_ROOT / "src" / "cases"
CASE_ID_PATTERN = re.compile(r"[a-z0-9][a-z0-9-]*")
CARGO_PASS_PATTERN = re.compile(r"test result: ok\. (\d+) passed;")
ALLOWED_PREFIXES = ("cargo test ", "python3 ")


def _arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--case", required=True)
    return parser.parse_args()


def _case_constants(case_id: str) -> dict[str, object]:
    if CASE_ID_PATTERN.fullmatch(case_id) is None:
        raise RuntimeError(f"invalid claim case id: {case_id}")
    path = CASE_ROOT / f"{case_id}.py"
    if not path.is_file():
        raise RuntimeError(f"claim case module does not exist: {path}")
    wanted = {
        "CASE_ID",
        "CAPABILITY_ID",
        "USE_CASE_ID",
        "TARGET_COMMANDS",
        "CLAIM_ORACLE",
        "ASSERTIONS",
    }
    values: dict[str, object] = {}
    for node in ast.parse(path.read_text(encoding="utf-8"), filename=str(path)).body:
        if not isinstance(node, ast.Assign) or len(node.targets) != 1:
            continue
        target = node.targets[0]
        if isinstance(target, ast.Name) and target.id in wanted:
            values[target.id] = ast.literal_eval(node.value)
    missing = sorted(wanted - values.keys())
    if missing:
        raise RuntimeError(f"{path} is missing constants: {', '.join(missing)}")
    if values["CASE_ID"] != case_id:
        raise RuntimeError(f"{path} CASE_ID does not match {case_id}")
    commands = values["TARGET_COMMANDS"]
    assertions = values["ASSERTIONS"]
    if (
        not isinstance(commands, tuple)
        or not commands
        or not all(isinstance(command, str) and command for command in commands)
    ):
        raise RuntimeError(f"{path} TARGET_COMMANDS must be a non-empty string tuple")
    if (
        not isinstance(assertions, tuple)
        or len(assertions) < 2
        or not all(isinstance(assertion, str) and assertion for assertion in assertions)
    ):
        raise RuntimeError(f"{path} ASSERTIONS must contain concrete outcomes")
    if str(values["CLAIM_ORACLE"]).strip() not in assertions[0]:
        raise RuntimeError(f"{path} ASSERTIONS do not project CLAIM_ORACLE")
    return values


def _run_command(command: str) -> dict[str, object]:
    if not command.startswith(ALLOWED_PREFIXES):
        raise RuntimeError(f"unsupported claim gate command: {command}")
    completed = subprocess.run(
        shlex.split(command),
        cwd=REPOSITORY_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    output = completed.stdout + completed.stderr
    print(output, end="" if output.endswith("\n") else "\n")
    if completed.returncode != 0:
        raise RuntimeError(
            f"claim gate exited {completed.returncode}: {command}"
        )
    passed_tests = None
    if command.startswith("cargo test "):
        passed_tests = sum(
            int(count) for count in CARGO_PASS_PATTERN.findall(output)
        )
        if passed_tests == 0:
            raise RuntimeError(
                f"cargo test gate executed zero tests: {command}"
            )
    return {
        "command": command,
        "exit_code": completed.returncode,
        "passed_tests": passed_tests,
    }


def main() -> int:
    args = _arguments()
    case = _case_constants(args.case)
    results = [_run_command(command) for command in case["TARGET_COMMANDS"]]
    print(
        json.dumps(
            {
                "schema_version": "aw.python-ec.claim-gate.v1",
                "status": "passed",
                "case_id": args.case,
                "capability_id": case["CAPABILITY_ID"],
                "use_case_id": case["USE_CASE_ID"],
                "assertions": case["ASSERTIONS"],
                "gate_count": len(results),
                "results": results,
            },
            sort_keys=True,
        )
    )
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, SyntaxError, ValueError) as error:
        print(f"claim gate failed: {error}")
        raise SystemExit(1)
