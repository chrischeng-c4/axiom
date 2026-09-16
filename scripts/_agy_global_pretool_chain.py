#!/usr/bin/env python3
"""Run the reviewed cap hook, then the frozen executor guard loader.

AGY executes only the first command hook for a matching PreToolUse rule.  The
existing cap-agent-guard owns the more-specific ``run_command`` matcher, so a
single fixed wrapper preserves that guard and gives the assignment loader a
chance to record allowed requests.  No hook program is selected from an
environment variable or task-controlled path.
"""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path


CAP_HOOK = (str(Path.home() / ".local" / "bin" / "cap"), "hook", "agy")
LOADER_FILENAME = "agy-assignment-guard-loader.py"


def deny(reason: str) -> None:
    print(json.dumps({"decision": "deny", "reason": reason}, sort_keys=True))


def hook_decision(output: bytes, name: str) -> str | None:
    try:
        value = json.loads(output.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError):
        return None
    decision = value.get("decision") if isinstance(value, dict) else None
    return decision if decision in {"allow", "deny"} else None


def invoke(command: tuple[str, ...], payload: bytes) -> tuple[int, bytes, bytes]:
    completed = subprocess.run(
        command,
        input=payload,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    return completed.returncode, completed.stdout, completed.stderr


def main() -> None:
    payload = sys.stdin.buffer.read()
    cap_code, cap_stdout, cap_stderr = invoke(CAP_HOOK, payload)
    if cap_stderr:
        sys.stderr.buffer.write(cap_stderr)
    cap_decision = hook_decision(cap_stdout, "cap")
    if cap_code or cap_decision is None:
        deny("reviewed cap-agent guard returned an invalid hook response")
        return
    if cap_decision == "deny":
        sys.stdout.buffer.write(cap_stdout)
        return

    loader = str(Path(__file__).resolve().with_name(LOADER_FILENAME))
    loader_code, loader_stdout, loader_stderr = invoke((loader,), payload)
    if loader_stderr:
        sys.stderr.buffer.write(loader_stderr)
    loader_decision = hook_decision(loader_stdout, "assignment loader")
    if loader_code or loader_decision is None:
        deny("fixed assignment guard loader returned an invalid hook response")
        return
    sys.stdout.buffer.write(loader_stdout)


if __name__ == "__main__":
    main()
