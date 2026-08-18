from __future__ import annotations

import importlib.util
import json
import os
import sys
from pathlib import Path

SCHEMA_VERSION = "aw.python-artifact.result.v1"

_HERE = Path(__file__).resolve().parent
_ROOT = _HERE.parent
_DESIGN_SRC = _HERE.parents[1] / "tech-design" / "src"
if str(_DESIGN_SRC) not in sys.path:
    sys.path.insert(0, str(_DESIGN_SRC))


def _fail(
    msg: str,
    command: str,
    src_digest: str,
    dep_digest: str,
    evidence_dir_str: str,
) -> None:
    print(msg, file=sys.stderr)
    ev_file_name = command if command else "unknown"
    if evidence_dir_str:
        try:
            ev_dir = Path(evidence_dir_str)
            ev_dir.mkdir(parents=True, exist_ok=True)
            ev_path = ev_dir / f"{ev_file_name}.json"
            ev_payload = {
                "case_id": ev_file_name,
                "checks": [],
                "passed": False,
                "error": msg,
            }
            ev_path.write_text(json.dumps(ev_payload, indent=2), encoding="utf-8")
        except Exception:
            pass

    envelope = {
        "schema_version": SCHEMA_VERSION,
        "status": "failed",
        "source_digest": src_digest,
        "dependency_lock_digest": dep_digest,
        "evidence": [f"evidence/{ev_file_name}.json"],
    }
    print(json.dumps(envelope))
    sys.exit(1)


def main() -> None:
    command = sys.argv[1] if len(sys.argv) > 1 else ""
    protocol = os.environ.get("AW_PYTHON_ARTIFACT_PROTOCOL", "")
    src_digest = os.environ.get("AW_PYTHON_ARTIFACT_SOURCE_DIGEST", "")
    dep_digest = os.environ.get("AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST", "")
    evidence_dir_str = os.environ.get("AW_PYTHON_ARTIFACT_EVIDENCE_DIR", "")

    if protocol != "aw.python-artifact.v1":
        _fail(
            f"protocol mismatch: {protocol}",
            command,
            src_digest,
            dep_digest,
            evidence_dir_str,
        )

    if not command or not src_digest or not dep_digest or not evidence_dir_str:
        _fail(
            "missing required environment variables or command argument",
            command,
            src_digest,
            dep_digest,
            evidence_dir_str,
        )

    case_file = _HERE / f"{command}.py"
    if not case_file.is_file():
        _fail(
            f"unknown command or missing case file: {command}",
            command,
            src_digest,
            dep_digest,
            evidence_dir_str,
        )

    mod_name = command.replace("-", "_")
    spec = importlib.util.spec_from_file_location(mod_name, case_file)
    if spec is None or spec.loader is None:
        _fail(
            f"failed to load module spec for command: {command}",
            command,
            src_digest,
            dep_digest,
            evidence_dir_str,
        )

    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    try:
        spec.loader.exec_module(module)
    except Exception as exc:
        _fail(
            f"error executing module {command}: {exc}",
            command,
            src_digest,
            dep_digest,
            evidence_dir_str,
        )

    func_name = f"verify_{mod_name}"
    func = getattr(module, func_name, None)
    if func is None or not callable(func):
        _fail(
            f"verifier function {func_name} not found in {case_file}",
            command,
            src_digest,
            dep_digest,
            evidence_dir_str,
        )

    try:
        result = func()
    except Exception as exc:
        result = {
            "case_id": command,
            "checks": [],
            "passed": False,
            "error": f"{type(exc).__name__}: {exc}",
        }

    evidence_dir = Path(evidence_dir_str)
    evidence_dir.mkdir(parents=True, exist_ok=True)
    evidence_path = evidence_dir / f"{command}.json"
    evidence_path.write_text(json.dumps(result, indent=2), encoding="utf-8")

    status = "passed" if result.get("passed") is True else "failed"

    envelope = {
        "schema_version": SCHEMA_VERSION,
        "status": status,
        "source_digest": src_digest,
        "dependency_lock_digest": dep_digest,
        "evidence": [f"evidence/{command}.json"],
    }
    print(json.dumps(envelope))
    sys.exit(0 if status == "passed" else 1)


if __name__ == "__main__":
    main()
