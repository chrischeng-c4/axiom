from __future__ import annotations

from pathlib import Path
import sys
from typing import Any, Iterable

_HERE = Path(__file__).resolve().parent
_DESIGN_SRC = _HERE.parents[1] / "tech-design" / "src"
if str(_DESIGN_SRC) not in sys.path:
    sys.path.insert(0, str(_DESIGN_SRC))

from build_stamp.application.emit_stamp import StampRequest, StampService
from build_stamp.domain.fallback import UNKNOWN
from build_stamp.domain.target import decode_target


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

BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX = (
    ("a_failed_git_invocation_degrades_only_the_commit_stamp", ("unknown", "1700000000", "aarch64-apple-darwin")),
    ("an_empty_git_answer_degrades_only_the_commit_stamp", ("unknown", "1700000000", "aarch64-apple-darwin")),
    ("an_absent_target_triple_degrades_only_the_target_stamp", ("c3ff13cd", "1700000000", "unknown")),
    ("an_unreadable_clock_degrades_only_the_build_time_stamp", ("c3ff13cd", "unknown", "aarch64-apple-darwin")),
    ("a_clock_before_the_epoch_degrades_only_the_build_time_stamp", ("c3ff13cd", "unknown", "aarch64-apple-darwin")),
    ("two_simultaneous_failures_degrade_exactly_those_two_stamps", ("unknown", "unknown", "aarch64-apple-darwin")),
    ("three_simultaneous_failures_degrade_all_three_stamps", ("unknown", "unknown", "unknown")),
    ("the_directive_count_is_the_same_under_every_failure_row", True),
    ("the_three_degraded_values_are_one_value_not_three_similar_ones", True),
    ("the_fallback_word_is_the_one_the_promise_names", "unknown"),
    ("a_degraded_commit_stamp_keeps_its_declared_directive_name", "LUMEN_GIT_SHA"),
    ("the_rerun_hint_is_unaffected_by_any_value_failure", "cargo:rerun-if-changed=../../.git/HEAD"),
    ("a_total_failure_still_renders_a_complete_directive_list", 3),
    ("an_absent_target_and_an_empty_target_are_not_the_same_case", ("unknown", "")),
)


def verify_best_effort_degradation_behavior() -> dict[str, Any]:
    good_sha = LocalShaSource(True, b"c3ff13cd\n")
    fail_sha = LocalShaSource(False, b"")
    empty_sha = LocalShaSource(True, b"")
    good_clock = LocalClockSource(1700000000)
    none_clock = LocalClockSource(None)
    neg_clock = LocalClockSource(-1)
    good_target = LocalTargetSource("aarch64-apple-darwin")
    none_target = LocalTargetSource(None)
    probe = LocalPathProbe({"../../.git/HEAD"})

    checks: list[dict[str, Any]] = []

    # 1. a_failed_git_invocation_degrades_only_the_commit_stamp
    exp1 = BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[0][1]
    p1 = StampService(fail_sha, good_clock, good_target, probe).plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs1 = tuple(d.value for d in p1.directives if d.key.startswith("LUMEN"))
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1,
    })

    # 2. an_empty_git_answer_degrades_only_the_commit_stamp
    exp2 = BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[1][1]
    p2 = StampService(empty_sha, good_clock, good_target, probe).plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs2 = tuple(d.value for d in p2.directives if d.key.startswith("LUMEN"))
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. an_absent_target_triple_degrades_only_the_target_stamp
    exp3 = BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[2][1]
    p3 = StampService(good_sha, good_clock, none_target, probe).plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs3 = tuple(d.value for d in p3.directives if d.key.startswith("LUMEN"))
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3,
    })

    # 4. an_unreadable_clock_degrades_only_the_build_time_stamp
    exp4 = BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[3][1]
    p4 = StampService(good_sha, none_clock, good_target, probe).plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs4 = tuple(d.value for d in p4.directives if d.key.startswith("LUMEN"))
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. a_clock_before_the_epoch_degrades_only_the_build_time_stamp
    exp5 = BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[4][1]
    p5 = StampService(good_sha, neg_clock, good_target, probe).plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs5 = tuple(d.value for d in p5.directives if d.key.startswith("LUMEN"))
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. two_simultaneous_failures_degrade_exactly_those_two_stamps
    exp6 = BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[5][1]
    p6 = StampService(fail_sha, none_clock, good_target, probe).plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs6 = tuple(d.value for d in p6.directives if d.key.startswith("LUMEN"))
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. three_simultaneous_failures_degrade_all_three_stamps
    exp7 = BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[6][1]
    p7 = StampService(fail_sha, none_clock, none_target, probe).plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs7 = tuple(d.value for d in p7.directives if d.key.startswith("LUMEN"))
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # 8. the_directive_count_is_the_same_under_every_failure_row
    exp8 = BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[7][1]
    plans = [p1, p2, p3, p4, p5, p6, p7]
    obs8 = all(len(p.directives) == len(p1.directives) for p in plans)
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. the_three_degraded_values_are_one_value_not_three_similar_ones
    exp9 = BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[8][1]
    v_sha, v_clock, v_target = obs7
    obs9 = (v_sha == v_clock == v_target == UNKNOWN)
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    # 10. the_fallback_word_is_the_one_the_promise_names
    exp10 = BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[9][1]
    obs10 = UNKNOWN
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. a_degraded_commit_stamp_keeps_its_declared_directive_name
    exp11 = BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[10][1]
    obs11 = next(d.key for d in p1.directives if d.key == "LUMEN_GIT_SHA")
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. the_rerun_hint_is_unaffected_by_any_value_failure
    exp12 = BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[11][1]
    obs12 = p7.render()[0]
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    # 13. a_total_failure_still_renders_a_complete_directive_list
    exp13 = BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[12][1]
    p7_no_hint = StampService(fail_sha, none_clock, none_target, LocalPathProbe(frozenset())).plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs13 = len(p7_no_hint.render())
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[12][0],
        "expected": exp13,
        "observed": obs13,
        "passed": obs13 == exp13,
    })

    # 14. an_absent_target_and_an_empty_target_are_not_the_same_case
    exp14 = BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[13][1]
    obs14 = (decode_target(None), decode_target(""))
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_BEHAVIOR_MATRIX[13][0],
        "expected": exp14,
        "observed": obs14,
        "passed": obs14 == exp14,
    })

    return {
        "case_id": "best-effort-degradation-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
