from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import sys

# Insert tech-design/src onto sys.path (tech-design and external-contracts are siblings under libs/metrics-prometheus)
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "tech-design" / "src"))


def main() -> None:
    protocol = os.environ.get("AW_PYTHON_ARTIFACT_PROTOCOL")
    source_digest = os.environ.get("AW_PYTHON_ARTIFACT_SOURCE_DIGEST")
    lock_digest = os.environ.get("AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST")
    evidence_dir = os.environ.get("AW_PYTHON_ARTIFACT_EVIDENCE_DIR")

    if len(sys.argv) < 2:
        _fail_exit("Missing command argument", source_digest, lock_digest)

    command = sys.argv[1]

    if protocol != "aw.python-artifact.v1" or not source_digest or not lock_digest or not evidence_dir:
        _fail_exit(f"Invalid or missing environment variables for command {command}", source_digest, lock_digest, command)

    case_file = Path(__file__).resolve().parent / f"{command}.py"
    if not case_file.exists():
        _fail_exit(f"Case file {case_file} does not exist", source_digest, lock_digest, command)

    func_name = f"verify_{command.replace('-', '_')}"

    try:
        spec = importlib.util.spec_from_file_location(f"case_{command.replace('-', '_')}", case_file)
        if spec is None or spec.loader is None:
            raise RuntimeError(f"Could not load spec for {case_file}")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)

        verify_fn = getattr(module, func_name, None)
        if verify_fn is None or not callable(verify_fn):
            raise RuntimeError(f"Function {func_name} not found in {case_file}")

        result = verify_fn()
    except Exception as exc:
        sys.stderr.write(f"Execution error: {exc}\n")
        result = {
            "case_id": command,
            "minimum_checks": 14,
            "checks": [],
            "passed": False,
            "error": str(exc),
        }

    evidence_path = Path(evidence_dir) / f"{command}.json"
    evidence_path.parent.mkdir(parents=True, exist_ok=True)
    with open(evidence_path, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2)

    passed = result.get("passed") is True
    status = "passed" if passed else "failed"

    out = {
        "schema_version": "aw.python-artifact.result.v1",
        "status": status,
        "source_digest": source_digest or "",
        "dependency_lock_digest": lock_digest or "",
        "evidence": [f"evidence/{command}.json"],
    }
    sys.stdout.write(json.dumps(out) + "\n")
    sys.exit(0 if passed else 1)


def _fail_exit(msg: str, source_digest: str | None, lock_digest: str | None, command: str = "unknown") -> None:
    sys.stderr.write(f"Runner failure: {msg}\n")
    out = {
        "schema_version": "aw.python-artifact.result.v1",
        "status": "failed",
        "source_digest": source_digest or "",
        "dependency_lock_digest": lock_digest or "",
        "evidence": [f"evidence/{command}.json"],
    }
    sys.stdout.write(json.dumps(out) + "\n")
    sys.exit(1)


if __name__ == "__main__":
    main()
