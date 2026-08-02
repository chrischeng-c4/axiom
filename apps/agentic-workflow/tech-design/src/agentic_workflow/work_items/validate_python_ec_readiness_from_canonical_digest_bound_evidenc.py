"Tech design for WI #3325: aw: validate Python EC readiness from canonical digest-bound evidence.\n\n@spec #3325"

from __future__ import annotations

import hashlib
import json
import os
import tempfile
from dataclasses import dataclass
from pathlib import Path
from typing import Union


__aw_artifact_id__ = "artifact:capability-control-plane/validate-python-ec-readiness-from-canonical-digest-bound-evidenc-wi-3325"
__aw_work_item__ = "3325"

__aw_changes__ = """
## Changes

```yaml
changes:
  - path: apps/agentic-workflow/src/services/python_artifact_readiness.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Parse and fail-close canonical digest-bound Python EC evidence before projecting readiness."
```
"""

EVIDENCE_PROTOCOL = "aw.python-ec.evidence.v1"


@dataclass(frozen=True)
class EvidenceRecord:
    protocol: str; case_id: str; mode: str; exit_code: int
    attempts: tuple[dict, ...]; assertions: tuple[str, ...] | None
    implementation: str | None; implementation_digest: str | None
    source_digest: str | None; declared_command: str | None; command: str | None


@dataclass(frozen=True)
class LoadedRecord:
    data: dict


class LoadKind:
    MISSING_EMPTY = "missing_empty"
    UNREADABLE = "unreadable"
    NOT_JSON = "not_json"


@dataclass(frozen=True)
class LoadError:
    kind: str
    blocker: str


LoadResult = Union[LoadedRecord, LoadError]


def load_evidence_file(path: Path) -> LoadResult:
    """LoadedRecord on success; LoadError for symlinks, non-regular, empty, unreadable, or unparseable.

    Non-dict JSON parses to LoadedRecord with _type sentinel; schema binding rejects it (design point 9).
    """
    if path.is_symlink() or not path.exists() or not path.is_file():
        return LoadError(kind=LoadKind.MISSING_EMPTY, blocker=f"evidence `{path}` is missing or empty")
    try:
        raw = path.read_bytes()
    except OSError:
        return LoadError(kind=LoadKind.UNREADABLE, blocker=f"evidence `{path}` is unreadable")
    if not raw.strip():
        return LoadError(kind=LoadKind.MISSING_EMPTY, blocker=f"evidence `{path}` is missing or empty")
    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError:
        return LoadError(kind=LoadKind.NOT_JSON, blocker=f"evidence `{path}` is not valid JSON")
    try:
        parsed = json.loads(text)
    except json.JSONDecodeError:
        return LoadError(kind=LoadKind.NOT_JSON, blocker=f"evidence `{path}` is not valid JSON")
    return LoadedRecord(data=parsed if isinstance(parsed, dict) else {"_type": type(parsed).__name__})


def _sha256(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def _assertions_digest(assertions: list[str]) -> str:
    return _sha256(json.dumps(assertions, ensure_ascii=True, separators=(",", ":")).encode())


@dataclass(frozen=True)
class BindingResult:
    record: EvidenceRecord | None
    blocker: str | None


def bind_evidence_record(
    raw: dict,
    *,
    case_id: str,
    evidence_rel_path: str,
    current_source_digest: str,
    declared_command: str,
    ec_root: Path,
    expected_implementation: str,
) -> BindingResult:
    """Fail-closed binding: every check must pass before returning a ready record."""
    prefix = f"Python EC case `{case_id}` evidence `{evidence_rel_path}`"

    if not isinstance(raw, dict) or "_type" in raw:
        return BindingResult(None, f"{prefix} is not valid JSON")
    if raw.get("protocol") != EVIDENCE_PROTOCOL:
        return BindingResult(None, f"{prefix} has unsupported protocol")
    if raw.get("case_id") != case_id:
        return BindingResult(None, f"{prefix} names case `{raw.get('case_id')}`")

    # Exact integer exit_code; booleans rejected
    ec = raw.get("exit_code")
    if isinstance(ec, bool) or type(ec) is not int or ec != 0:
        return BindingResult(None, f"{prefix} does not record successful execution")

    attempts = raw.get("attempts")
    if not isinstance(attempts, list) or not attempts:
        return BindingResult(None, f"{prefix} has no attempt records")

    # Common identity binding — applies before mode branching
    if raw.get("source_digest") != current_source_digest:
        return BindingResult(None, f"{prefix} is stale for the current source digest")
    if raw.get("declared_command") != declared_command:
        return BindingResult(None, f"{prefix} does not match the declared command")

    # Normalize and verify implementation identity; fail closed on any mismatch
    impl_path = ec_root / expected_implementation
    if raw.get("implementation") != expected_implementation:
        return BindingResult(None, f"{prefix} is stale for `{expected_implementation}`")
    if impl_path.is_symlink() or not impl_path.is_file():
        return BindingResult(None, f"{prefix} is stale for `{expected_implementation}`")
    try:
        current_impl_digest = _sha256(impl_path.read_bytes())
    except OSError:
        return BindingResult(None, f"{prefix} is stale for `{expected_implementation}`")
    if raw.get("implementation_digest") != current_impl_digest:
        return BindingResult(None, f"{prefix} is stale for `{expected_implementation}`")

    assertions: list[str] | None = raw.get("assertions")

    if assertions is not None:
        # Python-verifier path
        if not isinstance(assertions, list) or not assertions or not all(isinstance(a, str) and a for a in assertions):
            return BindingResult(None, f"{prefix} records zero executed assertions or tests")
        for attempt in attempts:
            if not isinstance(attempt, dict):
                return BindingResult(None, f"{prefix} does not record successful execution")
            ac = attempt.get("exit_code")
            if isinstance(ac, bool) or type(ac) is not int or ac != 0:
                return BindingResult(None, f"{prefix} does not record successful execution")
            cnt = attempt.get("assertion_count")
            if isinstance(cnt, bool) or not isinstance(cnt, int):
                return BindingResult(None, f"{prefix} has non-integer assertion_count")
            if cnt != len(assertions):
                return BindingResult(None, f"{prefix} has wrong assertion_count")
            ad = attempt.get("assertions_digest")
            if ad is not None:
                if not isinstance(ad, str):
                    return BindingResult(None, f"{prefix} has invalid assertions_digest type")
                if ad != _assertions_digest(assertions):
                    return BindingResult(None, f"{prefix} has wrong assertions_digest")
    else:
        # External-test-runner path
        for attempt in attempts:
            if not isinstance(attempt, dict):
                return BindingResult(None, f"{prefix} does not record successful execution")
            ac = attempt.get("exit_code")
            if isinstance(ac, bool) or type(ac) is not int or ac != 0:
                return BindingResult(None, f"{prefix} does not record successful execution")
            pt = attempt.get("passed_tests")
            if isinstance(pt, bool) or not isinstance(pt, int) or pt <= 0:
                return BindingResult(None, f"{prefix} records zero executed assertions or tests")
            ft = attempt.get("failed_tests")
            if isinstance(ft, bool) or not isinstance(ft, int) or ft != 0:
                return BindingResult(None, f"{prefix} records zero executed assertions or tests")

    return BindingResult(
        record=EvidenceRecord(
            protocol=raw["protocol"], case_id=case_id, mode=raw.get("mode", ""),
            exit_code=ec, attempts=tuple(attempts),
            assertions=tuple(assertions) if assertions is not None else None,
            implementation=raw.get("implementation"),
            implementation_digest=raw.get("implementation_digest"),
            source_digest=raw.get("source_digest"),
            declared_command=raw.get("declared_command"),
            command=raw.get("command"),
        ),
        blocker=None,
    )


@dataclass(frozen=True)
class CaseEvidenceReadiness:
    case_id: str
    evidence_ready: bool
    blocker: str | None


def evaluate_case_evidence(
    *,
    case_id: str,
    evidence_path: Path,
    evidence_rel_path: str,
    current_source_digest: str,
    declared_command: str,
    ec_root: Path,
    expected_implementation: str,
) -> CaseEvidenceReadiness:
    """Single shared projection for capability report and health spec (R3).

    Preserves the exact LoadError blocker. Missing/empty yields the protected
    contract wording `missing or empty digest-bound evidence`; malformed
    yields the exact `is not valid JSON` blocker from the loader.
    """
    load_result = load_evidence_file(evidence_path)
    if isinstance(load_result, LoadError):
        if load_result.kind == LoadKind.MISSING_EMPTY:
            blocker = f"Python EC case `{case_id}` has missing or empty digest-bound evidence"
        elif load_result.kind == LoadKind.UNREADABLE:
            blocker = f"Python EC case `{case_id}` evidence `{evidence_rel_path}` is unreadable"
        else:
            blocker = f"Python EC case `{case_id}` evidence `{evidence_rel_path}` is not valid JSON"
        return CaseEvidenceReadiness(case_id=case_id, evidence_ready=False, blocker=blocker)
    binding = bind_evidence_record(
        load_result.data, case_id=case_id, evidence_rel_path=evidence_rel_path,
        current_source_digest=current_source_digest, declared_command=declared_command,
        ec_root=ec_root, expected_implementation=expected_implementation,
    )
    if binding.blocker is not None:
        return CaseEvidenceReadiness(case_id=case_id, evidence_ready=False, blocker=binding.blocker)
    return CaseEvidenceReadiness(case_id=case_id, evidence_ready=True, blocker=None)


def remediation_command(project: str, *, applicability: str) -> str:
    """Return the canonical remediation command for a not-ready case.

    `applicability` is the EC inventory case applicability field (`td` or `cb`).
    """
    return f"aw ec verify --project {project} --stage {applicability}"


def design_contract() -> str:
    """Executable probes for all binding, loader, and projection contracts."""
    CMD = "uv run --frozen --offline --project . python src/runner.py --case demo-readiness"
    IMPL = "src/cases/readiness.py"

    with tempfile.TemporaryDirectory() as tmp:
        d = Path(tmp)
        impl_file = d / IMPL
        impl_file.parent.mkdir(parents=True)
        impl_file.write_bytes(b"def verify(): return ['ok']")
        impl_digest = _sha256(impl_file.read_bytes())

        # --- Loader probes ---
        r = load_evidence_file(d / "missing.json")
        assert isinstance(r, LoadError) and "missing or empty" in r.blocker, r

        e = d / "empty.json"; e.write_bytes(b"")
        r = load_evidence_file(e)
        assert isinstance(r, LoadError) and "missing or empty" in r.blocker, r

        bad = d / "bad.json"; bad.write_bytes(b"{not json")
        r = load_evidence_file(bad)
        assert isinstance(r, LoadError) and "not valid JSON" in r.blocker, r

        sym = d / "sym.json"; os.symlink(d / "missing_target.json", sym)
        r = load_evidence_file(sym)
        assert isinstance(r, LoadError) and "missing or empty" in r.blocker, r

        vf = d / "v.json"; vf.write_text('{"protocol":"x"}', encoding="utf-8")
        r = load_evidence_file(vf)
        assert isinstance(r, LoadedRecord) and r.data == {"protocol": "x"}, r

        sf = d / "s.json"; sf.write_text('"string"', encoding="utf-8")
        r = load_evidence_file(sf)
        assert isinstance(r, LoadedRecord), r

        # --- evaluate_case_evidence preserves loader blocker ---
        cer = evaluate_case_evidence(case_id="demo-readiness", evidence_path=d / "no.json",
            evidence_rel_path="evidence/no.json", current_source_digest="s",
            declared_command=CMD, ec_root=d, expected_implementation=IMPL)
        assert not cer.evidence_ready and "missing or empty digest-bound evidence" in cer.blocker, cer

        (d / "evidence").mkdir()
        mf = d / "evidence/mal.json"; mf.write_bytes(b"{bad")
        cer2 = evaluate_case_evidence(case_id="demo-readiness", evidence_path=mf,
            evidence_rel_path="evidence/mal.json", current_source_digest="s",
            declared_command=CMD, ec_root=d, expected_implementation=IMPL)
        assert not cer2.evidence_ready and "not valid JSON" in cer2.blocker, cer2

        # Collision-path probe: path containing "missing or empty" must still project not_json
        col = d / "evidence/missing or empty-trap.json"; col.write_bytes(b"{bad json")
        cer3 = evaluate_case_evidence(case_id="demo-readiness", evidence_path=col,
            evidence_rel_path="evidence/missing or empty-trap.json", current_source_digest="s",
            declared_command=CMD, ec_root=d, expected_implementation=IMPL)
        assert not cer3.evidence_ready and "not valid JSON" in cer3.blocker, cer3
        assert "missing or empty digest-bound evidence" not in cer3.blocker, cer3

        # --- Helpers ---
        def _rec(**kw: object) -> dict:
            base: dict = {"protocol": EVIDENCE_PROTOCOL, "case_id": "demo-readiness", "mode": "behavior",
                "exit_code": 0, "attempts": [{"exit_code": 0, "assertion_count": 1}],
                "assertions": ["ok"], "implementation": IMPL, "implementation_digest": impl_digest,
                "source_digest": "sha256:src", "declared_command": CMD}
            base.update(kw); return base

        bkw = dict(case_id="demo-readiness", evidence_rel_path="e/r.json",
            current_source_digest="sha256:src", declared_command=CMD, ec_root=d,
            expected_implementation=IMPL)

        def _ext(**kw: object) -> dict:
            base: dict = {"protocol": EVIDENCE_PROTOCOL, "case_id": "demo-readiness", "mode": "behavior",
                "exit_code": 0, "attempts": [{"exit_code": 0, "passed_tests": 3, "failed_tests": 0}],
                "implementation": IMPL, "implementation_digest": impl_digest,
                "source_digest": "sha256:src", "declared_command": CMD, "command": CMD}
            base.update(kw); return base

        # --- Python-verifier: assertions_digest / count probes ---
        correct_digest = _assertions_digest(["ok"])
        assert bind_evidence_record(_rec(), **bkw).blocker is None  # missing digest + count 1 → valid
        # present correct digest on attempt → valid
        b = bind_evidence_record(_rec(attempts=[{"exit_code": 0, "assertion_count": 1, "assertions_digest": correct_digest}]), **bkw)
        assert b.blocker is None, f"present-correct-valid: {b.blocker}"
        b = bind_evidence_record(_rec(attempts=[{"exit_code": 0, "assertion_count": 2}]), **bkw)
        assert b.blocker and "wrong assertion_count" in b.blocker, b  # positive wrong count → rejected
        b = bind_evidence_record(_rec(attempts=[{"exit_code": 0, "assertion_count": True}]), **bkw)
        assert b.blocker and "non-integer assertion_count" in b.blocker, b  # boolean count → rejected
        # present wrong digest on attempt → rejected
        b = bind_evidence_record(_rec(attempts=[{"exit_code": 0, "assertion_count": 1, "assertions_digest": "sha256:wrong"}]), **bkw)
        assert b.blocker and "wrong assertions_digest" in b.blocker, b

        # Two-attempt stability: each attempt independently valid
        two = _rec(attempts=[{"exit_code": 0, "assertion_count": 1}, {"exit_code": 0, "assertion_count": 1}])
        assert bind_evidence_record(two, **bkw).blocker is None

        # --- remediation_command: applicability-bound, exact-command probe ---
        assert remediation_command("demo", applicability="td") == "aw ec verify --project demo --stage td"
        assert remediation_command("demo", applicability="cb") == "aw ec verify --project demo --stage cb"

        # --- Python-verifier R2 negative controls ---
        for mutation, frag in [
            ({"protocol": "aw.python-ec.evidence.v0"}, "unsupported protocol"),
            ({"case_id": "other"}, "names case `other`"),
            ({"exit_code": 1, "attempts": [{"exit_code": 1, "assertion_count": 1}]}, "successful execution"),
            ({"exit_code": True}, "successful execution"),
            ({"assertions": [], "attempts": [{"exit_code": 0, "assertion_count": 0}]}, "zero executed"),
            ({"declared_command": "other"}, "declared command"),
            ({"source_digest": "sha256:stale"}, "stale for the current source digest"),
            ({"implementation": "src/cases/other.py"}, f"stale for `{IMPL}`"),
            ({"implementation_digest": "sha256:wrong"}, f"stale for `{IMPL}`"),
            ({"attempts": [{"exit_code": True, "assertion_count": 1}]}, "successful execution"),
        ]:
            b = bind_evidence_record(_rec(**mutation), **bkw)
            assert b.blocker and frag in b.blocker, (mutation, b.blocker)

        # --- External-test-runner probes ---
        assert bind_evidence_record(_ext(), **bkw).blocker is None
        for mutation, frag in [
            ({"source_digest": "sha256:stale"}, "stale for the current source digest"),
            ({"declared_command": "other"}, "declared command"),
            ({"attempts": [{"exit_code": 1, "passed_tests": 3, "failed_tests": 0}]}, "successful execution"),
            ({"attempts": [{"exit_code": True, "passed_tests": 3, "failed_tests": 0}]}, "successful execution"),
            ({"attempts": [{"exit_code": 0, "passed_tests": 0, "failed_tests": 0}]}, "zero executed"),
            ({"attempts": [{"exit_code": 0, "passed_tests": True, "failed_tests": 0}]}, "zero executed"),
            ({"attempts": [{"exit_code": 0, "passed_tests": 3, "failed_tests": 1}]}, "zero executed"),
        ]:
            b = bind_evidence_record(_ext(**mutation), **bkw)
            assert b.blocker and frag in b.blocker, (mutation, b.blocker)

    return (
        "design_contract:validate_python_ec_readiness_from_canonical_digest_bound_evidence:"
        "loader=ok,projection_preservation=ok,common_identity=ok,"
        "python_verifier=ok,external_runner=ok,r2_negative_controls=ok"
    )
