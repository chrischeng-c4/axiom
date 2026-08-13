"""Protocol runner for the lumen Python external contracts.

``--list`` is purely descriptive.  Case execution reads supplied TD or retained
GKE evidence and writes an EC result; it never invokes a cloud or harness
command.  A controller runs the real GKE harness separately, then binds this
runner to its generated run id, source revision, and freshness boundary.

Two routes exist.  The #2879 GKE contract predates this project's row-shaped
case convention and keeps the hardcoded entry points in ``run_case``.  Every
case authored since is one self-contained file named for its case id, resolved
by name in ``_run_generic_case`` so a new contract costs no branch here.
"""

from __future__ import annotations

from datetime import datetime
import importlib.util
import json
import os
import sys
from pathlib import Path
from typing import Any, Callable


RESULT_SCHEMA = "aw.python-artifact.result.v1"
PROTOCOL = "aw.python-artifact.v1"
TD_CASE_IDS = ("gke-ksa-rbac-td-behavior", "gke-ksa-rbac-td-security")
CB_CASE_IDS = ("gke-ksa-rbac-cb-behavior", "gke-ksa-rbac-cb-security")
CASE_IDS = TD_CASE_IDS + CB_CASE_IDS
CASE_EVIDENCE_PATHS = {case_id: f"evidence/{case_id}.json" for case_id in CASE_IDS}

# A row-shaped case imports the design it verifies. The TD source root is
# reached from this artifact's own location for the same reason `_repo_root`
# refuses an environment override: an arbitrary checkout must not be able to
# redirect verification at attacker-selected source.
_DESIGN_SRC = Path(__file__).resolve().parents[2] / "tech-design" / "src"

# Lumen's design composes shared-crate designs the way the crate does at
# runtime, so a lumen module may delegate a decision to the crate that owns it
# rather than restating it. Those designs are separate Python projects, and the
# contract cannot import through `lumen.*` unless they are importable too.
#
# Enumerated, not globbed over `libs/*/tech-design/src`: this is exactly the
# set of libs Lumen already declares as path dependencies in
# `apps/lumen/Cargo.toml`, so the design-time import surface is the twin of the
# compile-time one. A glob would drift from that set the moment a lib exists
# that Lumen does not depend on, and would then let an import be satisfied by a
# crate the implementation could not use.
#
# The two lists are kept equal by assertion, not by discipline -- a narrower
# hand-picked subset is a second list with no principle selecting its members,
# and it fails late, inside a worker that cannot import a crate for a reason
# that has nothing to do with its round.
_REPO_ROOT = Path(__file__).resolve().parents[4]
_SHARED_DESIGN_CRATES = (
    "build-stamp",
    "cli-std",
    "metrics-prometheus",
    "openapi-codegen",
    "peer-tls",
    "raft-core",
    "raft-runtime",
    "service-auth",
    "service-backup",
    "service-http",
    "service-k8s",
    "service-observability",
    "storage-durable",
    "transport-h2c",
)
_SHARED_DESIGN_SRC = tuple(
    _REPO_ROOT / "libs" / crate / "tech-design" / "src"
    for crate in _SHARED_DESIGN_CRATES
)

# Lumen's own root goes on last so it is searched first: a shared crate must
# never be able to shadow a `lumen.*` module the contract names.
for _root in (*_SHARED_DESIGN_SRC, _DESIGN_SRC):
    if _root.is_dir() and str(_root) not in sys.path:
        sys.path.insert(0, str(_root))


def _required_env(name: str) -> str:
    value = os.environ.get(name)
    if not value:
        raise ValueError(f"missing required environment variable {name}")
    return value


def _load_case() -> Any:
    path = Path(__file__).with_name("ec-2879.py")
    spec = importlib.util.spec_from_file_location("lumen_ec_2879", path)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load the #2879 external-contract module")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def _parse_not_before(value: str) -> datetime:
    module = _load_case()
    return module._parse_timestamp(value, "controller freshness boundary")


def _write_evidence(
    evidence_dir: Path,
    case_id: str,
    status: str,
    source_digest: str,
    dependency_lock_digest: str,
    detail: str,
) -> str:
    evidence_dir.mkdir(parents=True, exist_ok=True)
    relative = CASE_EVIDENCE_PATHS[case_id]
    result = {
        "schema": "axiom.lumen.ec.2879.case-result.v1",
        "case_id": case_id,
        "status": status,
        "source_digest": source_digest,
        "dependency_lock_digest": dependency_lock_digest,
        "detail": detail,
        "evidence_paths": [relative],
    }
    (evidence_dir / Path(relative).name).write_text(
        json.dumps(result, sort_keys=True) + "\n", encoding="utf-8"
    )
    return relative


def _emit(
    status: str,
    source_digest: str | None,
    dependency_lock_digest: str | None,
    cases: list[dict[str, str]],
) -> None:
    result: dict[str, Any] = {
        "schema_version": RESULT_SCHEMA,
        "status": status,
        "cases": cases,
        "evidence": [case["evidence_path"] for case in cases if "evidence_path" in case],
    }
    if source_digest is not None:
        result["source_digest"] = source_digest
    if dependency_lock_digest is not None:
        result["dependency_lock_digest"] = dependency_lock_digest
    print(json.dumps(result, sort_keys=True))


def _list_cases() -> None:
    _emit(
        "listed",
        None,
        None,
        [
            {
                "id": case_id,
                "applicability": "td" if case_id in TD_CASE_IDS else "cb",
                "evidence_path": CASE_EVIDENCE_PATHS[case_id],
            }
            for case_id in CASE_IDS
        ],
    )


def _repo_root() -> Path:
    """Resolve the TD root from this protocol-owned artifact location only.

    There is deliberately no environment override: an arbitrary checkout could
    otherwise make production TD verification inspect attacker-selected source.
    Tests place a copied artifact in a synthetic repository layout instead.
    """
    return Path(__file__).resolve().parents[4]


def run_case(case_id: str, source_digest: str, dependency_lock_digest: str, evidence_dir: Path) -> str:
    module = _load_case()
    if case_id == "gke-ksa-rbac-td-behavior":
        module.verify_td_behavior_source(
            _repo_root(),
            _required_env("LUMEN_EC_EXPECTED_TD_SOURCE_DIGEST"),
        )
    elif case_id == "gke-ksa-rbac-td-security":
        module.verify_td_security_source(
            _repo_root(),
            _required_env("LUMEN_EC_EXPECTED_TD_SOURCE_DIGEST"),
        )
    elif case_id == "gke-ksa-rbac-cb-behavior":
        module.verify_cb_behavior_evidence(
            Path(_required_env("LUMEN_EC_RETAINED_BUNDLE")),
            _required_env("LUMEN_EC_EXPECTED_RUN_ID"),
            _required_env("LUMEN_EC_EXPECTED_GIT_SHA"),
            _parse_not_before(_required_env("LUMEN_EC_NOT_BEFORE")),
            _required_env("LUMEN_EC_EXPECTED_SOURCE_ARCHIVE_COMMITMENT"),
            _required_env("LUMEN_EC_EXPECTED_GCP_PROJECT"),
            _required_env("LUMEN_EC_EXPECTED_REDACTION_COMMITMENT"),
            _required_env("LUMEN_EC_EXPECTED_GOOGLE_USER_PRINCIPAL"),
            _required_env("LUMEN_EC_EXPECTED_GOOGLE_SERVICE_ACCOUNT_PRINCIPAL"),
            _required_env("LUMEN_EC_EXPECTED_ISSUER_CHALLENGE"),
            _required_env("LUMEN_EC_EXPECTED_ATTESTATION_DSSE_DIGEST"),
            _required_env("LUMEN_EC_TRUSTED_CLOUDBUILD_ED25519_PUBLIC_KEY"),
        )
    elif case_id == "gke-ksa-rbac-cb-security":
        module.verify_cb_security_evidence(
            Path(_required_env("LUMEN_EC_RETAINED_BUNDLE")),
            _required_env("LUMEN_EC_EXPECTED_RUN_ID"),
            _required_env("LUMEN_EC_EXPECTED_GIT_SHA"),
            _parse_not_before(_required_env("LUMEN_EC_NOT_BEFORE")),
            _required_env("LUMEN_EC_EXPECTED_SOURCE_ARCHIVE_COMMITMENT"),
            _required_env("LUMEN_EC_EXPECTED_GCP_PROJECT"),
            _required_env("LUMEN_EC_EXPECTED_REDACTION_COMMITMENT"),
            _required_env("LUMEN_EC_EXPECTED_GOOGLE_USER_PRINCIPAL"),
            _required_env("LUMEN_EC_EXPECTED_GOOGLE_SERVICE_ACCOUNT_PRINCIPAL"),
            _required_env("LUMEN_EC_EXPECTED_ISSUER_CHALLENGE"),
            _required_env("LUMEN_EC_EXPECTED_CLI_BINARY_DIGEST"),
            _required_env("LUMEN_EC_EXPECTED_ATTESTATION_DSSE_DIGEST"),
            _required_env("LUMEN_EC_TRUSTED_CLOUDBUILD_ED25519_PUBLIC_KEY"),
            _required_env("LUMEN_EC_TRUSTED_CONTROLLER_ED25519_PUBLIC_KEY"),
        )
    else:
        raise ValueError(f"unknown external-contract command {case_id!r}")
    return _write_evidence(
        evidence_dir,
        case_id,
        "passed",
        source_digest,
        dependency_lock_digest,
        "EC-owned literal oracle accepted the supplied evidence",
    )


def _case_file(command: str) -> Path:
    return Path(__file__).with_name(f"{command}.py")


def _run_generic_case(command: str, evidence_dir: Path) -> str:
    """Route a row-shaped case to ``src/<command>.py::verify_<command>()``.

    The verifier owns its own result payload -- ``minimum_checks``, the
    ``checks`` rows, and ``passed`` -- and this function only persists it. A
    runner that recomputed ``passed`` from the rows would be a second opinion
    on the case's own verdict, and the two would eventually disagree.
    """
    case_file = _case_file(command)
    module_name = command.replace("-", "_")
    spec = importlib.util.spec_from_file_location(module_name, case_file)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"could not load external-contract module {case_file}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    verifier = getattr(module, f"verify_{module_name}", None)
    if not callable(verifier):
        raise ValueError(f"{case_file.name} defines no verify_{module_name}()")
    result = verifier()
    evidence_dir.mkdir(parents=True, exist_ok=True)
    (evidence_dir / f"{command}.json").write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    if result.get("passed") is not True:
        failing = [c.get("name") for c in result.get("checks", []) if not c.get("passed")]
        raise ValueError(f"case did not pass: {failing or 'no failing row reported'}")
    return f"evidence/{command}.json"


def _write_generic_failure(
    evidence_dir: Path, command: str, detail: str
) -> str:
    evidence_dir.mkdir(parents=True, exist_ok=True)
    payload = {"case_id": command, "checks": [], "passed": False, "detail": detail}
    path = evidence_dir / f"{command}.json"
    if path.is_file():
        # The verifier already wrote its rows; keep them and record why the
        # run failed. Overwriting here would erase the very rows that name the
        # failure and leave only this summary behind.
        existing = json.loads(path.read_text(encoding="utf-8"))
        existing["detail"] = detail
        payload = existing
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return f"evidence/{command}.json"


def main() -> None:
    command = sys.argv[1] if len(sys.argv) == 2 else ""
    if command == "--list":
        _list_cases()
        return
    source_digest = os.environ.get("AW_PYTHON_ARTIFACT_SOURCE_DIGEST")
    dependency_lock_digest = os.environ.get("AW_PYTHON_ARTIFACT_DEPENDENCY_LOCK_DIGEST")
    evidence_dir_value = os.environ.get("AW_PYTHON_ARTIFACT_EVIDENCE_DIR")
    generic = command not in CASE_IDS and bool(command) and _case_file(command).is_file()
    try:
        if os.environ.get("AW_PYTHON_ARTIFACT_PROTOCOL") != PROTOCOL:
            raise ValueError("AW_PYTHON_ARTIFACT_PROTOCOL is missing or unsupported")
        if command not in CASE_IDS and not generic:
            raise ValueError(f"unknown external-contract command {command!r}")
        if not source_digest or not dependency_lock_digest or not evidence_dir_value:
            raise ValueError("artifact source, dependency-lock, and evidence environment is required")
        if generic:
            evidence = _run_generic_case(command, Path(evidence_dir_value))
        else:
            evidence = run_case(command, source_digest, dependency_lock_digest, Path(evidence_dir_value))
    except Exception as error:
        have_env = bool(source_digest and dependency_lock_digest and evidence_dir_value)
        if have_env and generic:
            evidence = _write_generic_failure(Path(evidence_dir_value), command, str(error))
            _emit("failed", source_digest, dependency_lock_digest, [{"id": command, "evidence_path": evidence}])
        elif have_env and command in CASE_IDS:
            evidence = _write_evidence(
                Path(evidence_dir_value),
                command,
                "failed",
                source_digest,
                dependency_lock_digest,
                str(error),
            )
            _emit("failed", source_digest, dependency_lock_digest, [{"id": command, "evidence_path": evidence}])
        else:
            _emit("failed", source_digest, dependency_lock_digest, [{"id": command, "detail": str(error)}])
        raise SystemExit(1)
    _emit("passed", source_digest, dependency_lock_digest, [{"id": command, "evidence_path": evidence}])


if __name__ == "__main__":
    main()
