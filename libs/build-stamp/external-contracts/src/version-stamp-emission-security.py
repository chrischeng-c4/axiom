from __future__ import annotations

from pathlib import Path
import sys
from typing import Any, Iterable

_HERE = Path(__file__).resolve().parent
_DESIGN_SRC = _HERE.parents[1] / "tech-design" / "src"
if str(_DESIGN_SRC) not in sys.path:
    sys.path.insert(0, str(_DESIGN_SRC))

from build_stamp.application.emit_stamp import StampRequest, StampService
from build_stamp.domain.directive import DirectiveKind


class LocalShaSource:
    def __init__(self, success: bool, stdout: bytes) -> None:
        self._success = success
        self._stdout = stdout

    def read_short_sha(self) -> tuple[bool, bytes]:
        return self._success, self._stdout


class LocalClockSource:
    def __init__(self, seconds: int | None) -> None:
        self._seconds = seconds

    def epoch_seconds(self) -> int | None:
        return self._seconds


class LocalTargetSource:
    def __init__(self, target: str | None) -> None:
        self._target = target

    def target_triple(self) -> str | None:
        return self._target


class LocalPathProbe:
    def __init__(self, existing: Iterable[str] = ()) -> None:
        self._existing = frozenset(existing)

    def exists(self, path: str) -> bool:
        return path in self._existing


MINIMUM_CHECKS = 13

VERSION_STAMP_EMISSION_SECURITY_MATRIX = (
    ("no_emission_produces_a_line_from_a_forbidden_instruction_family", False),
    ("the_forbidden_family_check_would_notice_a_planted_line", True),
    ("the_emitted_directive_key_multiset_is_exactly_the_three_stamps", ("LUMEN_BUILT_AT", "LUMEN_GIT_SHA", "LUMEN_TARGET")),
    ("a_fourth_env_key_would_be_visible_in_the_key_multiset", 4),
    ("exactly_one_rerun_hint_is_emitted_when_the_path_exists", 1),
    ("exactly_zero_rerun_hints_are_emitted_when_the_path_is_absent", 0),
    ("the_directive_kind_enum_admits_only_the_two_declared_members", ("rerun-if-changed", "rustc-env")),
    ("every_rendered_line_starts_with_the_cargo_prefix", True),
    ("every_rendered_line_is_a_single_line", True),
    ("the_key_multiset_is_unchanged_by_an_unobtainable_input", ("LUMEN_BUILT_AT", "LUMEN_GIT_SHA", "LUMEN_TARGET")),
    ("the_key_multiset_is_unchanged_by_a_hostile_input", ("LUMEN_BUILT_AT", "LUMEN_GIT_SHA", "LUMEN_TARGET")),
    ("a_rendered_env_line_splits_into_exactly_three_parts_on_its_two_delimiters", (3, 3, 3)),
    ("the_stamp_emits_no_directive_beyond_the_declared_families", True),
)


def verify_version_stamp_emission_security() -> dict[str, Any]:
    forbidden_families = (
        "rustc-link-lib",
        "rustc-link-search",
        "rustc-link-arg",
        "rustc-cfg",
        "rustc-flags",
        "rustc-cdylib-link-arg",
        "warning",
        "error",
    )

    sha = LocalShaSource(True, b"c3ff13cd\n")
    clock = LocalClockSource(1700000000)
    target = LocalTargetSource("aarch64-apple-darwin")
    probe = LocalPathProbe({"../../.git/HEAD"})
    svc = StampService(sha, clock, target, probe)
    plan1 = svc.plan(StampRequest("LUMEN", "../../.git/HEAD"))
    rendered1 = plan1.render()

    checks: list[dict[str, Any]] = []

    # 1. no_emission_produces_a_line_from_a_forbidden_instruction_family
    exp1 = VERSION_STAMP_EMISSION_SECURITY_MATRIX[0][1]
    obs1 = any(any(line.startswith(f"cargo:{fam}") for fam in forbidden_families) for line in rendered1)
    checks.append({
        "name": VERSION_STAMP_EMISSION_SECURITY_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1,
    })

    # 2. the_forbidden_family_check_would_notice_a_planted_line
    exp2 = VERSION_STAMP_EMISSION_SECURITY_MATRIX[1][1]
    planted_lines = ["cargo:rustc-cfg=planted_test_line"]
    obs2 = any(any(line.startswith(f"cargo:{fam}") for fam in forbidden_families) for line in planted_lines)
    checks.append({
        "name": VERSION_STAMP_EMISSION_SECURITY_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. the_emitted_directive_key_multiset_is_exactly_the_three_stamps
    exp3 = VERSION_STAMP_EMISSION_SECURITY_MATRIX[2][1]
    obs3 = tuple(sorted(d.key for d in plan1.directives if d.key))
    checks.append({
        "name": VERSION_STAMP_EMISSION_SECURITY_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3,
    })

    # 4. a_fourth_env_key_would_be_visible_in_the_key_multiset
    exp4 = VERSION_STAMP_EMISSION_SECURITY_MATRIX[3][1]
    planted_multiset = list(obs3) + ["LUMEN_EXTRA"]
    obs4 = len(planted_multiset)
    checks.append({
        "name": VERSION_STAMP_EMISSION_SECURITY_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. exactly_one_rerun_hint_is_emitted_when_the_path_exists
    exp5 = VERSION_STAMP_EMISSION_SECURITY_MATRIX[4][1]
    obs5 = sum(1 for line in rendered1 if line.startswith("cargo:rerun-if-changed="))
    checks.append({
        "name": VERSION_STAMP_EMISSION_SECURITY_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. exactly_zero_rerun_hints_are_emitted_when_the_path_is_absent
    exp6 = VERSION_STAMP_EMISSION_SECURITY_MATRIX[5][1]
    svc_no_hint = StampService(sha, clock, target, LocalPathProbe(frozenset()))
    rendered_no_hint = svc_no_hint.plan(StampRequest("LUMEN", "../../.git/HEAD")).render()
    obs6 = sum(1 for line in rendered_no_hint if line.startswith("cargo:rerun-if-changed="))
    checks.append({
        "name": VERSION_STAMP_EMISSION_SECURITY_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. the_directive_kind_enum_admits_only_the_two_declared_members
    exp7 = VERSION_STAMP_EMISSION_SECURITY_MATRIX[6][1]
    obs7 = tuple(sorted(member.value for member in DirectiveKind))
    checks.append({
        "name": VERSION_STAMP_EMISSION_SECURITY_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # 8. every_rendered_line_starts_with_the_cargo_prefix
    exp8 = VERSION_STAMP_EMISSION_SECURITY_MATRIX[7][1]
    obs8 = all(line.startswith("cargo:") for line in rendered1)
    checks.append({
        "name": VERSION_STAMP_EMISSION_SECURITY_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. every_rendered_line_is_a_single_line
    exp9 = VERSION_STAMP_EMISSION_SECURITY_MATRIX[8][1]
    obs9 = not any("\n" in line or "\r" in line for line in rendered1)
    checks.append({
        "name": VERSION_STAMP_EMISSION_SECURITY_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    # 10. the_key_multiset_is_unchanged_by_an_unobtainable_input
    exp10 = VERSION_STAMP_EMISSION_SECURITY_MATRIX[9][1]
    svc_degraded = StampService(LocalShaSource(False, b""), LocalClockSource(None), LocalTargetSource(None), LocalPathProbe(frozenset()))
    plan_degraded = svc_degraded.plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs10 = tuple(sorted(d.key for d in plan_degraded.directives if d.key))
    checks.append({
        "name": VERSION_STAMP_EMISSION_SECURITY_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. the_key_multiset_is_unchanged_by_a_hostile_input
    exp11 = VERSION_STAMP_EMISSION_SECURITY_MATRIX[10][1]
    svc_hostile = StampService(LocalShaSource(True, b"\n\rcargo:rustc-cfg=x"), clock, target, probe)
    plan_hostile = svc_hostile.plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs11 = tuple(sorted(d.key for d in plan_hostile.directives if d.key))
    checks.append({
        "name": VERSION_STAMP_EMISSION_SECURITY_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. a_rendered_env_line_splits_into_exactly_three_parts_on_its_two_delimiters
    exp12 = VERSION_STAMP_EMISSION_SECURITY_MATRIX[11][1]
    env_lines = [line for line in rendered1 if line.startswith("cargo:rustc-env=")]
    obs12 = tuple(len(line.split("=", 2)) for line in env_lines)
    checks.append({
        "name": VERSION_STAMP_EMISSION_SECURITY_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    # 13. the_stamp_emits_no_directive_beyond_the_declared_families
    exp13 = VERSION_STAMP_EMISSION_SECURITY_MATRIX[12][1]
    allowed_prefixes = ("cargo:rustc-env=", "cargo:rerun-if-changed=")
    obs13 = all(any(line.startswith(p) for p in allowed_prefixes) for line in rendered1)
    checks.append({
        "name": VERSION_STAMP_EMISSION_SECURITY_MATRIX[12][0],
        "expected": exp13,
        "observed": obs13,
        "passed": obs13 == exp13,
    })

    return {
        "case_id": "version-stamp-emission-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
