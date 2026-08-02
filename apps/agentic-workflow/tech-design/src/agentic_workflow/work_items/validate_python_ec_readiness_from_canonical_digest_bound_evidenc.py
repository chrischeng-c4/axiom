"Tech design for WI #3325: aw: validate Python EC readiness from canonical digest-bound evidence.\n\n@spec #3325"

from __future__ import annotations

import hashlib
import json
import os
import tempfile
from dataclasses import dataclass
from enum import StrEnum
from pathlib import Path
from typing import Any


__aw_artifact_id__ = "artifact:capability-control-plane/validate-python-ec-readiness-from-canonical-digest-bound-evidenc-wi-3325"
__aw_work_item__ = "3325"


EVIDENCE_PROTOCOL = "aw.python-ec.evidence.v1"
REMEDIATION = "aw ec verify --project {project} --stage td"


class BlockerKind(StrEnum):
    MISSING_OR_EMPTY = "has missing or empty digest-bound evidence"
    NOT_JSON = "is not valid JSON"
    UNSUPPORTED_PROTOCOL = "has unsupported protocol"
    WRONG_CASE = "names case"
    STALE_SOURCE = "is stale for the current source digest"
    STALE_IMPLEMENTATION = "is stale for"
    COMMAND_MISMATCH = "does not match the declared command"
    NON_ZERO_EXIT = "does not record successful execution"
    ZERO_ORACLE = "records zero executed assertions or tests"


@dataclass(frozen=True)
class CanonicalEvidence:
    """Parsed aw.python-ec.evidence.v1. Verifier: assertions+assertion_count+assertions_digest. External-test: passed_tests/failed_tests. type(v) is int enforced."""

    protocol: str
    case_id: str
    source_digest: str
    declared_command: str
    implementation: str        # EC-root-relative; must equal case test_path
    implementation_digest: str
    exit_code: int
    assertions: tuple[str, ...]
    attempts: tuple[dict[str, Any], ...]


def blocker_message(
    *,
    case_id: str,
    evidence_path: str,
    kind: BlockerKind,
    detail: str | None = None,
) -> str:
    if kind == BlockerKind.MISSING_OR_EMPTY:
        return f"Python EC case `{case_id}` has missing or empty digest-bound evidence"
    prefix = f"Python EC case `{case_id}` evidence `{evidence_path}`"
    if kind == BlockerKind.WRONG_CASE:
        return f"{prefix} names case `{detail}`"
    if kind == BlockerKind.STALE_IMPLEMENTATION:
        return f"{prefix} is stale for `{detail}`"
    return f"{prefix} {kind.value}"


def _str_digest(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def _assertions_digest(assertions: list[str]) -> str:
    encoded = json.dumps(assertions, ensure_ascii=True, separators=(",", ":"))
    return "sha256:" + hashlib.sha256(encoded.encode()).hexdigest()


def _valid_attempt_verifier(a: Any, assertions: list[str]) -> bool:
    if not isinstance(a, dict):
        return False
    ec, ac = a.get("exit_code"), a.get("assertion_count")
    if type(ec) is not int or ec != 0:
        return False
    if type(ac) is not int or ac != len(assertions):
        return False
    ad = a.get("assertions_digest")
    # assertions_digest is mandatory; must be a string matching the canonical digest
    if not isinstance(ad, str) or ad != _assertions_digest(assertions):
        return False
    return True


def _valid_attempt_external(a: Any) -> bool:
    if not isinstance(a, dict):
        return False
    ec, pt, ft = a.get("exit_code"), a.get("passed_tests"), a.get("failed_tests")
    return (
        type(ec) is int and ec == 0
        and type(pt) is int and pt > 0
        and type(ft) is int and ft == 0
    )


def _has_oracle(raw: dict[str, Any]) -> bool:
    attempts = raw.get("attempts")
    if not isinstance(attempts, list) or not attempts:
        return False
    assertions = raw.get("assertions")
    if assertions is not None:
        # assertions present: must be a non-empty list of non-empty strings (verifier variant)
        if (
            not isinstance(assertions, list)
            or not assertions
            or not all(type(a) is str and a for a in assertions)
        ):
            return False
        return all(_valid_attempt_verifier(a, assertions) for a in attempts)
    # external-test variant: no assertions key
    return all(_valid_attempt_external(a) for a in attempts)


def parse_and_bind_evidence(
    raw: Any,
    *,
    expected_case_id: str,
    evidence_path: str,
    expected_source_digest: str,
    expected_declared_command: str,
    expected_implementation_digest: str,
    expected_implementation_path: str,
) -> CanonicalEvidence | str:
    """Fail-closed canonical parser bound to current contract inputs (R1, R2).

    Binding order: protocol → case_id → source_digest → implementation path
    → implementation digest → declared_command → exit_code → oracle.
    Returns a blocker string on the first failed check.
    """

    def _b(kind: BlockerKind, detail: str | None = None) -> str:
        return blocker_message(
            case_id=expected_case_id,
            evidence_path=evidence_path,
            kind=kind,
            detail=detail,
        )

    if not isinstance(raw, dict):
        return _b(BlockerKind.NOT_JSON)
    if raw.get("protocol") != EVIDENCE_PROTOCOL:
        return _b(BlockerKind.UNSUPPORTED_PROTOCOL)
    if raw.get("case_id") != expected_case_id:
        return _b(BlockerKind.WRONG_CASE, detail=str(raw.get("case_id")))
    if raw.get("source_digest") != expected_source_digest:
        return _b(BlockerKind.STALE_SOURCE)
    # Implementation path must equal test_path; wrong path fails even if digest matches
    if raw.get("implementation") != expected_implementation_path:
        return _b(BlockerKind.STALE_IMPLEMENTATION, detail=expected_implementation_path)
    if raw.get("implementation_digest") != expected_implementation_digest:
        return _b(BlockerKind.STALE_IMPLEMENTATION, detail=expected_implementation_path)
    if raw.get("declared_command") != expected_declared_command:
        return _b(BlockerKind.COMMAND_MISMATCH)
    ec = raw.get("exit_code")
    if type(ec) is not int or ec != 0:
        return _b(BlockerKind.NON_ZERO_EXIT)
    if not _has_oracle(raw):
        return _b(BlockerKind.ZERO_ORACLE)

    assertions = raw.get("assertions") or []
    attempts = [a for a in raw.get("attempts", []) if isinstance(a, dict)]
    return CanonicalEvidence(
        protocol=EVIDENCE_PROTOCOL,
        case_id=expected_case_id,
        source_digest=str(raw["source_digest"]),
        declared_command=str(raw["declared_command"]),
        implementation=str(raw["implementation"]),
        implementation_digest=str(raw["implementation_digest"]),
        exit_code=0,
        assertions=tuple(assertions),
        attempts=tuple(attempts),
    )

@dataclass(frozen=True)
class _LoadOk:
    value: Any

@dataclass(frozen=True)
class _LoadErr:
    blocker: str

def load_evidence_file(evidence_file: Path, *, case_id: str, evidence_path: str) -> _LoadOk | _LoadErr:
    """Tagged loader: missing/empty/symlink/unreadable → _LoadErr(MISSING_OR_EMPTY); bad encoding/JSON → _LoadErr(NOT_JSON)."""
    def _err(kind: BlockerKind) -> _LoadErr:
        return _LoadErr(blocker_message(case_id=case_id, evidence_path=evidence_path, kind=kind))
    try:
        if not evidence_file.exists() or evidence_file.is_symlink() or not evidence_file.is_file():
            return _err(BlockerKind.MISSING_OR_EMPTY)
        if evidence_file.stat().st_size == 0:
            return _err(BlockerKind.MISSING_OR_EMPTY)
        raw_bytes = evidence_file.read_bytes()
    except OSError:
        return _err(BlockerKind.MISSING_OR_EMPTY)
    try:
        text = raw_bytes.decode("utf-8")
    except (UnicodeDecodeError, ValueError):
        return _err(BlockerKind.NOT_JSON)
    try:
        return _LoadOk(json.loads(text))
    except json.JSONDecodeError:
        return _err(BlockerKind.NOT_JSON)


def load_and_bind_evidence(evidence_file: Path, *, case_id: str, evidence_path: str, expected_source_digest: str, expected_declared_command: str, expected_implementation_digest: str, expected_implementation_path: str) -> CanonicalEvidence | str:
    """Load from disk then bind; loader blockers pass through verbatim."""
    r = load_evidence_file(evidence_file, case_id=case_id, evidence_path=evidence_path)
    if isinstance(r, _LoadErr):
        return r.blocker
    return parse_and_bind_evidence(r.value, expected_case_id=case_id, evidence_path=evidence_path, expected_source_digest=expected_source_digest, expected_declared_command=expected_declared_command, expected_implementation_digest=expected_implementation_digest, expected_implementation_path=expected_implementation_path)


def shared_projection_contract() -> dict[str, str]:
    """One PythonArtifactReadiness value projected identically on both surfaces (R3)."""

    return {
        "producer": (
            "apps/agentic-workflow/src/services/python_artifact_readiness.rs::evaluate"
        ),
        "capability_report_caller": "apps/agentic-workflow/src/cli/capability.rs",
        "health_spec_caller": (
            "apps/agentic-workflow/src/cli/project.rs"
            "::apply_python_artifact_readiness_to_report"
        ),
        "remediation_command": REMEDIATION,
    }


def ownership_and_flow() -> dict[str, str]:
    return {
        "apps/agentic-workflow/external-contracts/src/runner.py":
            "canonical evidence emitter (R4)",
        "apps/agentic-workflow/src/services/python_artifact_readiness.rs":
            "fail-closed parser and binder (R1, R2)",
        "apps/agentic-workflow/src/cli/capability.rs":
            "shared projection caller (R3)",
        "apps/agentic-workflow/src/cli/project.rs::apply_python_artifact_readiness_to_report":
            "shared projection caller (R3)",
        "apps/agentic-workflow/external-contracts/src/cases/"
        "capability-control-plane-python-artifact-readiness.py":
            "focused acceptance and negative controls",
    }


def design_contract() -> str:
    """Express the executable design contract for this bounded change."""

    projection = shared_projection_contract()
    ownership = ownership_and_flow()

    # Both surfaces must project the same PythonArtifactReadiness value
    assert "apply_python_artifact_readiness_to_report" in projection["health_spec_caller"]
    assert projection["capability_report_caller"].endswith("capability.rs")
    assert projection["remediation_command"] == REMEDIATION
    assert len(ownership) == 5

    # ---- fixtures ----
    _case = "demo-readiness"
    _ep = "evidence/readiness.json"
    _src = "sha256:aaa"
    _impl_path = "src/cases/readiness.py"
    _impl_digest = _str_digest(b"def verify(): return ['ok']\n")
    _cmd = "uv run --frozen --offline --project . python src/runner.py --case demo-readiness"

    def _bind(raw: Any, impl_path: str = _impl_path) -> CanonicalEvidence | str:
        return parse_and_bind_evidence(
            raw,
            expected_case_id=_case,
            evidence_path=_ep,
            expected_source_digest=_src,
            expected_declared_command=_cmd,
            expected_implementation_digest=_impl_digest,
            expected_implementation_path=impl_path,
        )

    _good_digest = _assertions_digest(["ok"])
    _v = {
        "protocol": EVIDENCE_PROTOCOL, "case_id": _case,
        "source_digest": _src, "declared_command": _cmd,
        "implementation": _impl_path, "implementation_digest": _impl_digest,
        "exit_code": 0, "assertions": ["ok"],
        "attempts": [{"exit_code": 0, "assertion_count": 1, "assertions_digest": _good_digest}],
    }
    _ext = {
        "protocol": EVIDENCE_PROTOCOL, "case_id": _case,
        "source_digest": _src, "declared_command": _cmd,
        "implementation": _impl_path, "implementation_digest": _impl_digest,
        "exit_code": 0,
        "attempts": [{"exit_code": 0, "passed_tests": 3, "failed_tests": 0}],
    }

    # Happy paths: verifier and external-test variants
    assert isinstance(_bind(_v), CanonicalEvidence)
    assert isinstance(_bind(_ext), CanonicalEvidence)

    # bool must not pass as int
    assert isinstance(_bind({**_v, "exit_code": False}), str)
    assert isinstance(_bind({**_v, "attempts": [{"exit_code": 0, "assertion_count": True, "assertions_digest": _good_digest}]}), str)
    # verifier oracle: wrong count, wrong digest, missing digest each rejected
    assert isinstance(_bind({**_v, "attempts": [{"exit_code": 0, "assertion_count": 2, "assertions_digest": _good_digest}]}), str)
    _bad_digest = "sha256:" + "0" * 64
    assert isinstance(_bind({**_v, "attempts": [{"exit_code": 0, "assertion_count": 1, "assertions_digest": _bad_digest}]}), str)
    assert isinstance(_bind({**_v, "attempts": [{"exit_code": 0, "assertion_count": 1}]}), str)
    # malformed assertions rejected
    assert isinstance(_bind({**_ext, "assertions": [42]}), str)
    assert isinstance(_bind({**_v, "assertions": [""]}), str)

    # ---- load_and_bind boundary probes ----
    def _lab(file: Path) -> CanonicalEvidence | str:
        return load_and_bind_evidence(
            file, case_id=_case, evidence_path=_ep,
            expected_source_digest=_src, expected_declared_command=_cmd,
            expected_implementation_digest=_impl_digest,
            expected_implementation_path=_impl_path,
        )
    _mb = blocker_message(case_id=_case, evidence_path=_ep, kind=BlockerKind.MISSING_OR_EMPTY)
    _jb = blocker_message(case_id=_case, evidence_path=_ep, kind=BlockerKind.NOT_JSON)
    with tempfile.TemporaryDirectory() as _tmp:
        _d = Path(_tmp)
        _ef = _d / "readiness.json"
        # missing → exact MISSING_OR_EMPTY through combined boundary
        assert _lab(_ef) == _mb
        # empty, symlink → MISSING_OR_EMPTY
        _ef.write_bytes(b"")
        assert _lab(_ef) == _mb
        _real = _d / "real.json"; _real.write_text("{}", encoding="utf-8")
        _sym = _d / "link.json"; os.symlink(_real, _sym)
        assert _lab(_sym) == _mb
        # bad encoding/JSON → NOT_JSON (loader blocker, not schema blocker)
        _ef.write_bytes(b"\xff\xfe"); assert _lab(_ef) == _jb
        _ef.write_text("{bad", encoding="utf-8"); assert _lab(_ef) == _jb
        # valid JSON string content reaches schema binding → NOT_JSON from parse step
        _ef.write_text('"a string"', encoding="utf-8")
        assert _lab(_ef) == _jb
        # valid regular canonical JSON → CanonicalEvidence
        _ef.write_text(json.dumps(_v), encoding="utf-8")
        assert isinstance(_lab(_ef), CanonicalEvidence)
        # loader _LoadOk wraps dict, not str; tagged result prevents string-value confusion
        _r = load_evidence_file(_ef, case_id=_case, evidence_path=_ep)
        assert isinstance(_r, _LoadOk) and isinstance(_r.value, dict)

    # R2 negative controls (all via _bind)
    _z = BlockerKind.ZERO_ORACLE
    assert _bind("bad") == blocker_message(case_id=_case, evidence_path=_ep, kind=BlockerKind.NOT_JSON)
    assert _bind({**_v, "protocol": "aw.python-ec.evidence.v0"}) == blocker_message(case_id=_case, evidence_path=_ep, kind=BlockerKind.UNSUPPORTED_PROTOCOL)
    assert _bind({**_v, "case_id": "other"}) == blocker_message(case_id=_case, evidence_path=_ep, kind=BlockerKind.WRONG_CASE, detail="other")
    assert _bind({**_v, "source_digest": "sha256:stale"}) == blocker_message(case_id=_case, evidence_path=_ep, kind=BlockerKind.STALE_SOURCE)
    assert _bind(_v, impl_path="src/cases/other.py") == blocker_message(case_id=_case, evidence_path=_ep, kind=BlockerKind.STALE_IMPLEMENTATION, detail="src/cases/other.py")
    assert _bind({**_v, "implementation_digest": "sha256:old"}) == blocker_message(case_id=_case, evidence_path=_ep, kind=BlockerKind.STALE_IMPLEMENTATION, detail=_impl_path)
    assert _bind({**_v, "declared_command": "other"}) == blocker_message(case_id=_case, evidence_path=_ep, kind=BlockerKind.COMMAND_MISMATCH)
    assert _bind({**_v, "exit_code": 1, "attempts": [{"exit_code": 1, "assertion_count": 1, "assertions_digest": _good_digest}]}) == blocker_message(case_id=_case, evidence_path=_ep, kind=BlockerKind.NON_ZERO_EXIT)
    assert _bind({**_v, "assertions": [], "attempts": [{"exit_code": 0, "assertion_count": 0, "assertions_digest": _assertions_digest([])}]}) == blocker_message(case_id=_case, evidence_path=_ep, kind=_z)
    assert blocker_message(case_id=_case, evidence_path=_ep, kind=BlockerKind.MISSING_OR_EMPTY) == (
        "Python EC case `demo-readiness` has missing or empty digest-bound evidence"
    )

    return (
        "wi-3325 canonical digest-bound readiness: protocol="
        + EVIDENCE_PROTOCOL
        + " variants=verifier,external-test"
        + " binding=path+digest+source+command+exit+oracle"
        + " projection=shared(capability+health-spec)"
    )
