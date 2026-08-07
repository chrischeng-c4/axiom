from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import re
import secrets
import subprocess
import sys
from pathlib import Path
from typing import Any, Callable


RESULT_SCHEMA = "aw.python-artifact.result.v1"
PROTOCOL = "aw.python-artifact.v1"
CASE_ID = "gke-ksa-rbac-authorization"
EVIDENCE_NAME = "gke-ksa-rbac-authorization.json"
RUNS_DIR = "gke-runs"
AUTH_AUDIT_NAME = "lumen-auth-redaction-audit.json"
REDACTION_LIFECYCLE_FUNCTION = "lumen_auth_redaction_audit_and_destroy"
REDACTION_LIFECYCLE = re.compile(
    r"(?ms)^lumen_auth_redaction_audit_and_destroy\(\) \{\n"
    r"\s+\"\$\{LUMEN_AUTH_REDACTION_AUDITOR:\?[^}]+\}\" \\\n"
    r"\s+--evidence-root \"\$EVIDENCE_DIR\" \\\n"
    r"\s+--credential-dir \"\$SECRET_DIR\" \\\n"
    r"\s+--output \"\$\{LUMEN_AUTH_REDACTION_AUDIT_PATH:\?[^}]+\}\"\n"
    r"\s+rm -rf \"\$SECRET_DIR\"\n"
    r"\s+SECRET_DIR=\"\"\n"
    r"\}$"
)
REDACTION_LIFECYCLE_GUARD = re.compile(
    r"(?ms)^if \[\[ -n \"\$\{LUMEN_AUTH_REDACTION_AUDITOR:-\}\" \|\| -n \"\$\{LUMEN_AUTH_REDACTION_AUDIT_PATH:-\}\" \]\]; then\n"
    r"\s+\[\[ -n \"\$\{LUMEN_AUTH_REDACTION_AUDITOR:-\}\" && -n \"\$\{LUMEN_AUTH_REDACTION_AUDIT_PATH:-\}\" \]\] \|\| \{\n"
    r"\s+echo \"LUMEN_AUTH_REDACTION_AUDITOR and LUMEN_AUTH_REDACTION_AUDIT_PATH must be set together\" >&2\n"
    r"\s+exit 1\n"
    r"\s+\}\n"
    r"\s+lumen_auth_redaction_audit_and_destroy\n"
    r"fi$"
)


def _required_env(name: str) -> str:
    value = os.environ.get(name)
    if not value:
        raise ValueError(f"missing required environment variable {name}")
    return value


def _load_case() -> Callable[..., None]:
    case_path = Path(__file__).with_name("ec-2879.py")
    spec = importlib.util.spec_from_file_location("lumen_ec_2879", case_path)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load the #2879 external-contract case")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    verifier = getattr(module, "verify_retained_gke_evidence", None)
    if not callable(verifier):
        raise RuntimeError("#2879 external-contract case exports no retained-evidence verifier")
    return verifier


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[4]


def _current_checkout(repo_root: Path) -> str:
    dirty = subprocess.run(
        ["git", "-C", str(repo_root), "status", "--porcelain=v1"],
        check=True,
        capture_output=True,
        text=True,
    )
    if dirty.stdout:
        raise ValueError("GKE oracle requires a clean checkout for source provenance")
    revision = subprocess.run(
        ["git", "-C", str(repo_root), "rev-parse", "--short=12", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if not revision:
        raise ValueError("could not determine the reviewed checkout revision")
    return revision


def _has_ordered_harness_redaction_lifecycle(source: str) -> bool:
    lifecycle = REDACTION_LIFECYCLE.search(source)
    guard = REDACTION_LIFECYCLE_GUARD.search(source)
    if lifecycle is None or guard is None:
        return False
    without_blocks = (
        source[: lifecycle.start()]
        + source[lifecycle.end() : guard.start()]
        + source[guard.end() :]
    )
    return re.search(
        rf"(?m)^\s*{REDACTION_LIFECYCLE_FUNCTION}\s*$", without_blocks
    ) is None


def _require_harness_redaction_seam(script: Path) -> None:
    if not _has_ordered_harness_redaction_lifecycle(script.read_text(encoding="utf-8")):
        raise ValueError(
            "acceptance/gcp/scripts/verify-lumen-auth.sh lacks the required ordered "
            f"{REDACTION_LIFECYCLE_FUNCTION} lifecycle and strict opt-in pair guard"
        )


def _fresh_run_id() -> str:
    return f"ec2879{secrets.token_hex(6)}"


def _run_harness(script: Path, repo_root: Path, environment: dict[str, str]) -> None:
    completed = subprocess.run(
        [str(script)],
        cwd=repo_root,
        env=environment,
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise ValueError(f"fresh GKE acceptance harness failed with exit {completed.returncode}")


def _read_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as error:
        raise ValueError(f"missing fresh GKE artifact {path.name}") from error
    except json.JSONDecodeError as error:
        raise ValueError(f"invalid fresh GKE artifact {path.name}") from error
    if not isinstance(value, dict):
        raise ValueError(f"fresh GKE artifact {path.name} is not a JSON object")
    return value


def _verify_provenance(run_dir: Path, run_id: str, expected_sha: str) -> tuple[dict[str, Any], dict[str, Any]]:
    run = _read_json(run_dir / "run.json")
    if run.get("run_id") != run_id:
        raise ValueError("GKE run provenance does not carry the runner-generated run id")
    if run.get("git_sha") != expected_sha or run.get("git_dirty") is not False:
        raise ValueError("GKE run provenance does not match the current clean checkout")
    if not isinstance(run.get("started_at"), str) or not run["started_at"]:
        raise ValueError("GKE run provenance has no start time")
    if run.get("image_provenance") != "cloud-build":
        raise ValueError("GKE oracle requires a Lumen image built from the reviewed checkout")
    images = _read_json(run_dir / "images.json")
    image = images.get("lumen")
    if not isinstance(image, str) or "@sha256:" not in image:
        raise ValueError("GKE run did not retain an immutable Lumen image digest")
    return run, images


def _cloud_build_environment(
    project_id: str,
    run_id: str,
    run_dir: Path,
    audit_path: Path,
) -> dict[str, str]:
    environment = {
        **os.environ,
        "PROJECT_ID": project_id,
        "RUN_ID": run_id,
        "EVIDENCE_DIR": str(run_dir),
        "LUMEN_AUTH_REDACTION_AUDITOR": str(Path(__file__).with_name("redaction_auditor.py")),
        "LUMEN_AUTH_REDACTION_AUDIT_PATH": str(audit_path),
        "ACCEPTANCE_APPS": "lumen auth",
    }
    for override in (
        "LUMEN_IMAGE",
        "SIFT_IMAGE",
        "TAPE_IMAGE",
        "LUMEN_PRIOR_ACCEPTANCE",
        "LUMEN_CLI",
        "SIFT_CLI",
        "TAPE_CLI",
    ):
        environment.pop(override, None)
    return environment


def _digest(path: Path) -> str:
    return f"sha256:{hashlib.sha256(path.read_bytes()).hexdigest()}"


def _write_evidence(
    evidence_dir: Path,
    status: str,
    source_digest: str,
    dependency_lock_digest: str,
    detail: str,
    run_dir: Path | None,
    provenance: tuple[dict[str, Any], dict[str, Any]] | None = None,
) -> str:
    evidence_dir.mkdir(parents=True, exist_ok=True)
    result_path = evidence_dir / EVIDENCE_NAME
    result: dict[str, Any] = {
        "schema": "axiom.lumen.ec.gke-ksa-rbac.v1",
        "case": CASE_ID,
        "status": status,
        "source_digest": source_digest,
        "dependency_lock_digest": dependency_lock_digest,
        "detail": detail,
    }
    if run_dir is not None:
        auth = run_dir / "lumen-auth-acceptance.json"
        cleanup = run_dir / "cleanup.json"
        audit = run_dir / "kubernetes" / "auth" / AUTH_AUDIT_NAME
        if auth.is_file():
            result["auth_summary_digest"] = _digest(auth)
        if cleanup.is_file():
            result["cleanup_digest"] = _digest(cleanup)
        if audit.is_file():
            result["redaction_audit_digest"] = _digest(audit)
        if provenance is not None:
            run, images = provenance
            result["run_id"] = run["run_id"]
            result["git_sha"] = run["git_sha"]
            result["lumen_image"] = images["lumen"]
    result_path.write_text(json.dumps(result, sort_keys=True) + "\n", encoding="utf-8")
    return f"evidence/{EVIDENCE_NAME}"


def _emit(status: str, source_digest: str, dependency_lock_digest: str, evidence: str) -> None:
    print(
        json.dumps(
            {
                "schema_version": RESULT_SCHEMA,
                "status": status,
                "source_digest": source_digest,
                "dependency_lock_digest": dependency_lock_digest,
                "evidence": [evidence],
            },
            sort_keys=True,
        )
    )


def run_case(
    source_digest: str,
    dependency_lock_digest: str,
    evidence_dir: Path,
    project_id: str,
    run_harness: Callable[[Path, Path, dict[str, str]], None] = _run_harness,
    checkout: Callable[[Path], str] = _current_checkout,
) -> str:
    repo_root = _repo_root()
    script = repo_root / "acceptance" / "gcp" / "scripts" / "run.sh"
    auth_script = repo_root / "acceptance" / "gcp" / "scripts" / "verify-lumen-auth.sh"
    _require_harness_redaction_seam(auth_script)
    revision = checkout(repo_root)
    run_id = _fresh_run_id()
    run_dir = evidence_dir / RUNS_DIR / run_id
    run_dir.mkdir(parents=True, exist_ok=False)
    audit_path = run_dir / "kubernetes" / "auth" / AUTH_AUDIT_NAME
    environment = _cloud_build_environment(project_id, run_id, run_dir, audit_path)
    run_harness(script, repo_root, environment)
    provenance = _verify_provenance(run_dir, run_id, revision)
    _load_case()(run_dir, audit_path)
    return _write_evidence(
        evidence_dir,
        "passed",
        source_digest,
        dependency_lock_digest,
        "fresh GKE evidence satisfied the two-hop KSA/RBAC contract",
        run_dir,
        provenance,
    )


def main() -> None:
    source_digest = _required_env("AW_PYTHON_ARTIFACT_SOURCE_DIGEST")
    dependency_lock_digest = _required_env("AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST")
    evidence_dir = Path(_required_env("AW_PYTHON_ARTIFACT_EVIDENCE_DIR"))
    command = sys.argv[1] if len(sys.argv) == 2 else ""
    project_id = os.environ.get("PROJECT_ID")

    try:
        if os.environ.get("AW_PYTHON_ARTIFACT_PROTOCOL") != PROTOCOL:
            raise ValueError("AW_PYTHON_ARTIFACT_PROTOCOL is missing or unsupported")
        if command != CASE_ID:
            raise ValueError(f"unknown external-contract command {command!r}")
        if not project_id:
            raise ValueError("missing required environment variable PROJECT_ID")
        evidence = run_case(
            source_digest,
            dependency_lock_digest,
            evidence_dir,
            project_id,
        )
    except Exception as error:
        evidence = _write_evidence(
            evidence_dir,
            "failed",
            source_digest,
            dependency_lock_digest,
            str(error),
            None,
        )
        _emit("failed", source_digest, dependency_lock_digest, evidence)
        raise SystemExit(1)

    _emit("passed", source_digest, dependency_lock_digest, evidence)


if __name__ == "__main__":
    main()
