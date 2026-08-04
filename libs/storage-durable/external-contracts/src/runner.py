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

def fail_closed(msg: str, command: str = "", src_digest: str = "", dep_digest: str = "", evidence_dir: str = "") -> None:
    sys.stderr.write(f"{msg}\n")
    if evidence_dir and command:
        try:
            ev_dir = Path(evidence_dir)
            ev_dir.mkdir(parents=True, exist_ok=True)
            evidence_file = ev_dir / f"{command}.json"
            evidence_payload = {
                "case_id": command,
                "checks": [],
                "passed": False,
                "error": msg,
            }
            evidence_file.write_text(json.dumps(evidence_payload, indent=2))
        except Exception as err:
            sys.stderr.write(f"Failed to write evidence in fail_closed: {err}\n")

    result_env = {
        "schema_version": "aw.python-artifact.result.v1",
        "status": "failed",
        "source_digest": src_digest or "",
        "dependency_lock_digest": dep_digest or "",
        "evidence": [f"evidence/{command}.json"] if command else [],
    }
    sys.stdout.write(json.dumps(result_env) + "\n")
    sys.exit(1)

def main() -> None:
    protocol = os.environ.get("AW_PYTHON_ARTIFACT_PROTOCOL", "")
    src_digest = os.environ.get("AW_PYTHON_ARTIFACT_SOURCE_DIGEST", "")
    dep_digest = os.environ.get("AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST", "")
    evidence_dir = os.environ.get("AW_PYTHON_ARTIFACT_EVIDENCE_DIR", "")
    command = sys.argv[1] if len(sys.argv) > 1 else ""

    if protocol != "aw.python-artifact.v1":
        fail_closed(f"Invalid protocol: {protocol!r}", command, src_digest, dep_digest, evidence_dir)

    if not command or not src_digest or not dep_digest or not evidence_dir:
        fail_closed("Missing required command or environment variables", command, src_digest, dep_digest, evidence_dir)

    case_file = _HERE / f"{command}.py"
    if not case_file.is_file():
        fail_closed(f"Case file not found: {case_file}", command, src_digest, dep_digest, evidence_dir)

    module_name = command.replace("-", "_")
    spec = importlib.util.spec_from_file_location(module_name, case_file)
    if spec is None or spec.loader is None:
        fail_closed(f"Could not load spec for {case_file}", command, src_digest, dep_digest, evidence_dir)

    module = importlib.util.module_from_spec(spec)
    sys.modules[module_name] = module
    try:
        spec.loader.exec_module(module)
    except Exception as exc:
        fail_closed(f"Error loading module {module_name}: {exc}", command, src_digest, dep_digest, evidence_dir)

    func = getattr(module, f"verify_{module_name}", None)
    if func is None or not callable(func):
        fail_closed(f"Function verify_{module_name} missing or not callable", command, src_digest, dep_digest, evidence_dir)

    try:
        result = func()
    except Exception as exc:
        result = {
            "case_id": command,
            "checks": [],
            "passed": False,
            "error": f"{type(exc).__name__}: {exc}",
        }

    try:
        ev_dir = Path(evidence_dir)
        ev_dir.mkdir(parents=True, exist_ok=True)
        evidence_file = ev_dir / f"{command}.json"
        evidence_file.write_text(json.dumps(result, indent=2))
    except Exception as exc:
        fail_closed(f"Failed writing evidence file: {exc}", command, src_digest, dep_digest, evidence_dir)

    passed = result.get("passed") is True
    status = "passed" if passed else "failed"

    out_env = {
        "schema_version": "aw.python-artifact.result.v1",
        "status": status,
        "source_digest": src_digest,
        "dependency_lock_digest": dep_digest,
        "evidence": [f"evidence/{command}.json"],
    }
    sys.stdout.write(json.dumps(out_env) + "\n")
    sys.exit(0 if passed else 1)

if __name__ == "__main__":
    main()
