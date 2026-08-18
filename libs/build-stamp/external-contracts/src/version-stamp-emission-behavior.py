from __future__ import annotations

from pathlib import Path
import sys
from typing import Any, Iterable

_HERE = Path(__file__).resolve().parent
_DESIGN_SRC = _HERE.parents[1] / "tech-design" / "src"
if str(_DESIGN_SRC) not in sys.path:
    sys.path.insert(0, str(_DESIGN_SRC))

from build_stamp.application.emit_stamp import StampRequest, StampService
from build_stamp.domain.build_time import format_built_at


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


MINIMUM_CHECKS = 14

VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX = (
    ("the_three_env_directives_appear_in_the_declared_order", ("_GIT_SHA", "_BUILT_AT", "_TARGET")),
    ("each_directive_name_is_the_callers_prefix_joined_to_its_suffix", ("LUMEN_GIT_SHA", "LUMEN_BUILT_AT", "LUMEN_TARGET")),
    ("a_second_prefix_produces_the_same_shape_under_a_different_name", ("AXIOM_GIT_SHA", "AXIOM_BUILT_AT", "AXIOM_TARGET")),
    ("an_empty_prefix_still_produces_three_named_directives", ("_GIT_SHA", "_BUILT_AT", "_TARGET")),
    ("the_rendered_env_line_has_the_cargo_rustc_env_form", True),
    ("the_build_time_is_whole_seconds_since_the_epoch", "1700000000"),
    ("the_build_time_carries_no_unit_suffix_and_no_separator", True),
    ("the_epoch_itself_encodes_as_zero", "0"),
    ("the_rerun_hint_is_emitted_when_the_named_path_exists", "cargo:rerun-if-changed=../../.git/HEAD"),
    ("the_rerun_hint_is_absent_when_the_named_path_does_not_exist", False),
    ("the_rerun_hint_names_the_path_it_was_given", "cargo:rerun-if-changed=some/custom/path"),
    ("the_rerun_hint_precedes_every_env_directive", True),
    ("the_rendered_hint_line_has_the_cargo_rerun_if_changed_form", True),
    ("the_total_line_count_is_four_with_the_hint_and_three_without", (4, 3)),
)


def verify_version_stamp_emission_behavior() -> dict[str, Any]:
    sha = LocalShaSource(True, b"c3ff13cd\n")
    clock = LocalClockSource(1700000000)
    target = LocalTargetSource("aarch64-apple-darwin")
    probe = LocalPathProbe({"../../.git/HEAD"})
    svc = StampService(sha, clock, target, probe)

    checks: list[dict[str, Any]] = []

    # 1. the_three_env_directives_appear_in_the_declared_order
    exp1 = VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[0][1]
    plan1 = svc.plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs1 = tuple(d.key[len("LUMEN"):] for d in plan1.directives if d.key.startswith("LUMEN"))
    checks.append({
        "name": VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1,
    })

    # 2. each_directive_name_is_the_callers_prefix_joined_to_its_suffix
    exp2 = VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[1][1]
    obs2 = tuple(d.key for d in plan1.directives if d.key)
    checks.append({
        "name": VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. a_second_prefix_produces_the_same_shape_under_a_different_name
    exp3 = VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[2][1]
    plan3 = svc.plan(StampRequest("AXIOM", "../../.git/HEAD"))
    obs3 = tuple(d.key for d in plan3.directives if d.key)
    checks.append({
        "name": VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3,
    })

    # 4. an_empty_prefix_still_produces_three_named_directives
    exp4 = VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[3][1]
    plan4 = svc.plan(StampRequest("", "../../.git/HEAD"))
    obs4 = tuple(d.key for d in plan4.directives if d.key)
    checks.append({
        "name": VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. the_rendered_env_line_has_the_cargo_rustc_env_form
    exp5 = VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[4][1]
    rendered1 = plan1.render()
    obs5 = all(line.startswith("cargo:rustc-env=") for line in rendered1[1:])
    checks.append({
        "name": VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. the_build_time_is_whole_seconds_since_the_epoch
    exp6 = VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[5][1]
    obs6 = next(d.value for d in plan1.directives if d.key == "LUMEN_BUILT_AT")
    checks.append({
        "name": VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. the_build_time_carries_no_unit_suffix_and_no_separator
    exp7 = VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[6][1]
    obs7 = obs6.isdigit()
    checks.append({
        "name": VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # 8. the_epoch_itself_encodes_as_zero
    exp8 = VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[7][1]
    obs8 = format_built_at(0)
    checks.append({
        "name": VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. the_rerun_hint_is_emitted_when_the_named_path_exists
    exp9 = VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[8][1]
    obs9 = rendered1[0]
    checks.append({
        "name": VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    # 10. the_rerun_hint_is_absent_when_the_named_path_does_not_exist
    exp10 = VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[9][1]
    svc_no_hint = StampService(sha, clock, target, LocalPathProbe(frozenset()))
    rendered_no_hint = svc_no_hint.plan(StampRequest("LUMEN", "../../.git/HEAD")).render()
    obs10 = any(line.startswith("cargo:rerun-if-changed=") for line in rendered_no_hint)
    checks.append({
        "name": VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. the_rerun_hint_names_the_path_it_was_given
    exp11 = VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[10][1]
    svc_custom_path = StampService(sha, clock, target, LocalPathProbe({"some/custom/path"}))
    rendered_custom = svc_custom_path.plan(StampRequest("LUMEN", "some/custom/path")).render()
    obs11 = rendered_custom[0]
    checks.append({
        "name": VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. the_rerun_hint_precedes_every_env_directive
    exp12 = VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[11][1]
    obs12 = rendered1[0].startswith("cargo:rerun-if-changed=") and all(line.startswith("cargo:rustc-env=") for line in rendered1[1:])
    checks.append({
        "name": VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    # 13. the_rendered_hint_line_has_the_cargo_rerun_if_changed_form
    exp13 = VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[12][1]
    obs13 = rendered1[0].startswith("cargo:rerun-if-changed=")
    checks.append({
        "name": VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[12][0],
        "expected": exp13,
        "observed": obs13,
        "passed": obs13 == exp13,
    })

    # 14. the_total_line_count_is_four_with_the_hint_and_three_without
    exp14 = VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[13][1]
    obs14 = (len(rendered1), len(rendered_no_hint))
    checks.append({
        "name": VERSION_STAMP_EMISSION_BEHAVIOR_MATRIX[13][0],
        "expected": exp14,
        "observed": obs14,
        "passed": obs14 == exp14,
    })

    return {
        "case_id": "version-stamp-emission-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
