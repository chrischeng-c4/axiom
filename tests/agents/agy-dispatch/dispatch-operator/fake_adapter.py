#!/usr/bin/env python3
"""Local-only AGY adapter double for dispatch-operator model evals."""

from __future__ import annotations

import json
import os
import sys
import time
from pathlib import Path


EVAL_DIR = Path.cwd() / ".eval"
CONFIG = EVAL_DIR / "adapter-config.json"
TRACE = EVAL_DIR / "adapter-trace.jsonl"
LAUNCH_COMPLETE = EVAL_DIR / "launch-complete"
ALLOWED = {"doctor", "snapshot", "dispatch", "resume", "status"}


def load_config() -> dict[str, object]:
    if not CONFIG.is_file():
        return {}
    value = json.loads(CONFIG.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        raise SystemExit("adapter-config.json must contain an object")
    return value


def append_trace(record: dict[str, object]) -> None:
    EVAL_DIR.mkdir(parents=True, exist_ok=True)
    with TRACE.open("a", encoding="utf-8") as stream:
        stream.write(json.dumps(record, sort_keys=True) + "\n")


def launch_is_complete() -> bool:
    try:
        value = json.loads(LAUNCH_COMPLETE.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError):
        return False
    return isinstance(value, dict) and value.get("complete") is True


def main() -> int:
    verb = sys.argv[1] if len(sys.argv) > 1 else ""
    config = load_config()
    behavior = config.get(verb, {})
    if not isinstance(behavior, dict):
        raise SystemExit(f"adapter behavior for {verb!r} must be an object")

    exit_code = int(behavior.get("exit", 0))
    stdout = str(behavior.get("stdout", ""))
    stderr = str(behavior.get("stderr", ""))
    sleep_seconds = float(behavior.get("sleep_seconds", 0))

    if verb not in ALLOWED:
        exit_code = 97
        stderr = f"FORBIDDEN_ADAPTER_VERB: {verb}\n"
    elif verb == "status" and not launch_is_complete():
        exit_code = 4
        stdout = "STATUS_BEFORE_LAUNCH_COMPLETE\n"
    elif exit_code == 0 and not stdout:
        defaults = {
            "doctor": "DOCTOR_OK\n",
            "snapshot": "/tmp/agy-eval/snapshot.json\n",
            "dispatch": "FAKE_DISPATCH_PROCESS_COMPLETE\n",
            "resume": "FAKE_RESUME_PROCESS_COMPLETE\n",
            "status": "attempt: DELIVERED\nARTIFACT /tmp/agy-eval/report.md\n",
        }
        stdout = defaults.get(verb, "")

    append_trace(
        {
            "argv": sys.argv[1:],
            "cwd": str(Path.cwd().resolve()),
            "entrypoint": str(Path(__file__).resolve()),
            "exit": exit_code,
            "pid": os.getpid(),
            "stderr": stderr,
            "stdout": stdout,
            "verb": verb,
        }
    )

    if sleep_seconds:
        time.sleep(sleep_seconds)
    if verb in {"dispatch", "resume"}:
        LAUNCH_COMPLETE.write_text(
            json.dumps(
                {"complete": True, "exit": exit_code, "verb": verb},
                sort_keys=True,
            )
            + "\n",
            encoding="utf-8",
        )

    if stdout:
        sys.stdout.write(stdout)
        sys.stdout.flush()
    if stderr:
        sys.stderr.write(stderr)
        sys.stderr.flush()
    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
