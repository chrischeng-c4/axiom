from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import sys

_HERE = Path(__file__).resolve().parent
_DESIGN_SRC = _HERE.parents[1] / "tech-design" / "src"
if str(_DESIGN_SRC) not in sys.path:
    sys.path.insert(0, str(_DESIGN_SRC))


def _fail_closed(
    msg: str,
    command: str | None = None,
    evidence_dir: str | None = None,
    src_digest: str = "",
    dep_digest: str = "",
) -> None:
    print(f"ERROR: {msg}", file=sys.stderr)

    if command and evidence_dir:
        ev_dir_path = Path(evidence_dir)
        ev_dir_path.mkdir(parents=True, exist_ok=True)
        ev_file = ev_dir_path / f"{command}.json"
        res = {
            "case_id": command,
            "minimum_checks": 0,
            "checks": [],
            "passed": False,
            "error": msg,
        }
        ev_file.write_text(json.dumps(res, indent=2), encoding="utf-8")
        evidence_list = [f"evidence/{command}.json"]
    else:
        evidence_list = []

    envelope = {
        "schema_version": "aw.python-artifact.result.v1",
        "status": "failed",
        "source_digest": src_digest,
        "dependency_lock_digest": dep_digest,
        "evidence": evidence_list,
    }
    print(json.dumps(envelope))
    sys.exit(1)


def main() -> None:
    protocol = os.environ.get("AW_PYTHON_ARTIFACT_PROTOCOL", "")
    src_digest = os.environ.get("AW_PYTHON_ARTIFACT_SOURCE_DIGEST", "")
    dep_digest = os.environ.get("AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST", "")
    evidence_dir = os.environ.get("AW_PYTHON_ARTIFACT_EVIDENCE_DIR", "")
    command = sys.argv[1] if len(sys.argv) > 1 else ""

    if protocol != "aw.python-artifact.v1":
        _fail_closed(
            f"Invalid protocol: '{protocol}'",
            command,
            evidence_dir,
            src_digest,
            dep_digest,
        )

    if not command or not src_digest or not dep_digest or not evidence_dir:
        _fail_closed(
            "Missing required command or env var",
            command,
            evidence_dir,
            src_digest,
            dep_digest,
        )

    case_file = _HERE / f"{command}.py"
    if not case_file.is_file():
        _fail_closed(
            f"Case file not found: {case_file}",
            command,
            evidence_dir,
            src_digest,
            dep_digest,
        )

    module_name = command.replace("-", "_")
    func_name = f"verify_{module_name}"

    try:
        spec = importlib.util.spec_from_file_location(module_name, case_file)
        if spec is None or spec.loader is None:
            _fail_closed(
                f"Failed to load spec for {case_file}",
                command,
                evidence_dir,
                src_digest,
                dep_digest,
            )
            return
        mod = importlib.util.module_from_spec(spec)
        sys.modules[module_name] = mod
        spec.loader.exec_module(mod)
        func = getattr(mod, func_name, None)
        if func is None or not callable(func):
            _fail_closed(
                f"Function {func_name} missing or not callable",
                command,
                evidence_dir,
                src_digest,
                dep_digest,
            )
            return
    except Exception as exc:
        _fail_closed(
            f"Import error: {type(exc).__name__}: {exc}",
            command,
            evidence_dir,
            src_digest,
            dep_digest,
        )
        return

    try:
        result = func()
    except Exception as exc:
        result = {
            "case_id": command,
            "minimum_checks": getattr(mod, "MINIMUM_CHECKS", 0),
            "checks": [],
            "passed": False,
            "error": f"{type(exc).__name__}: {exc}",
        }

    ev_dir_path = Path(evidence_dir)
    ev_dir_path.mkdir(parents=True, exist_ok=True)
    ev_file = ev_dir_path / f"{command}.json"
    ev_file.write_text(json.dumps(result, indent=2), encoding="utf-8")

    passed = result.get("passed") is True
    status = "passed" if passed else "failed"

    envelope = {
        "schema_version": "aw.python-artifact.result.v1",
        "status": status,
        "source_digest": src_digest,
        "dependency_lock_digest": dep_digest,
        "evidence": [f"evidence/{command}.json"],
    }
    print(json.dumps(envelope))
    sys.exit(0 if passed else 1)


if __name__ == "__main__":
    main()
