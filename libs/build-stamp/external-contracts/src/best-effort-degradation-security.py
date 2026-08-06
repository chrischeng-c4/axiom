from __future__ import annotations

import dataclasses
import inspect
from pathlib import Path
import sys
from typing import Any, Iterable, get_type_hints

_HERE = Path(__file__).resolve().parent
_DESIGN_SRC = _HERE.parents[1] / "tech-design" / "src"
if str(_DESIGN_SRC) not in sys.path:
    sys.path.insert(0, str(_DESIGN_SRC))

from build_stamp.application import emit_stamp as emit_stamp_mod
from build_stamp.application.emit_stamp import StampPlan, StampRequest, StampService


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


MINIMUM_CHECKS = 12

BEST_EFFORT_DEGRADATION_SECURITY_MATRIX = (
    ("a_failed_git_invocation_leaks_no_part_of_its_output", False),
    ("a_failed_git_invocation_leaks_no_exit_status", False),
    ("an_absent_target_leaks_no_environment_variable_name", False),
    ("an_unreadable_clock_leaks_no_diagnostic_text", False),
    ("no_emitted_value_contains_a_filesystem_path_on_any_failure_row", False),
    ("the_poison_detector_would_notice_a_planted_payload", True),
    ("the_plan_result_type_carries_no_diagnostic_field", ("directives",)),
    ("the_entry_point_declares_a_total_return_type", "StampPlan"),
    ("the_entry_point_returns_a_plan_on_every_failure_row", True),
    ("every_failure_row_completes_without_raising", 4),
    ("a_degraded_value_is_the_fallback_word_and_nothing_appended_to_it", "unknown"),
    ("the_design_exposes_no_error_or_result_type_from_the_entry_point", ("StampPlan", "StampRequest", "StampService")),
)


def verify_best_effort_degradation_security() -> dict[str, Any]:
    poison_stderr = b"fatal: not a git repository"
    poison_exit = 128
    poison_env = "TARGET"
    poison_clock = "clock error: SystemTimeBeforeEpoch"
    poison_path = "/var/log/build.log"

    sha_fail = LocalShaSource(False, poison_stderr)
    clock_none = LocalClockSource(None)
    target_none = LocalTargetSource(None)
    probe_fail = LocalPathProbe({poison_path})

    svc = StampService(sha_fail, clock_none, target_none, LocalPathProbe(frozenset()))
    plan = svc.plan(StampRequest("LUMEN", "../../.git/HEAD"))
    rendered = plan.render()

    checks: list[dict[str, Any]] = []

    # 1. a_failed_git_invocation_leaks_no_part_of_its_output
    exp1 = BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[0][1]
    obs1 = any(poison_stderr.decode("utf-8") in line for line in rendered)
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1,
    })

    # 2. a_failed_git_invocation_leaks_no_exit_status
    exp2 = BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[1][1]
    obs2 = any(str(poison_exit) in line for line in rendered)
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. an_absent_target_leaks_no_environment_variable_name
    exp3 = BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[2][1]
    obs3 = any(d.value == poison_env for d in plan.directives)
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3,
    })

    # 4. an_unreadable_clock_leaks_no_diagnostic_text
    exp4 = BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[3][1]
    obs4 = any(poison_clock in line for line in rendered)
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. no_emitted_value_contains_a_filesystem_path_on_any_failure_row
    exp5 = BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[4][1]
    plan_path = StampService(sha_fail, clock_none, target_none, probe_fail).plan(StampRequest("LUMEN", poison_path))
    rendered_path = plan_path.render()
    env_values = [d.value for d in plan_path.directives if d.key]
    obs5 = any(poison_path in val for val in env_values)
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. the_poison_detector_would_notice_a_planted_payload
    exp6 = BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[5][1]
    planted_lines = ["cargo:rustc-env=K=" + poison_path]
    obs6 = any(poison_path in line for line in planted_lines)
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. the_plan_result_type_carries_no_diagnostic_field
    exp7 = BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[6][1]
    obs7 = tuple(f.name for f in dataclasses.fields(StampPlan))
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # 8. the_entry_point_declares_a_total_return_type
    exp8 = BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[7][1]
    hints = get_type_hints(StampService.plan)
    ret_type = hints.get("return", None)
    obs8 = ret_type.__name__ if hasattr(ret_type, "__name__") else str(ret_type)
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. the_entry_point_returns_a_plan_on_every_failure_row
    exp9 = BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[8][1]
    obs9 = isinstance(plan, StampPlan) and isinstance(plan_path, StampPlan)
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    # 10. every_failure_row_completes_without_raising
    exp10 = BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[9][1]
    degraded_configurations = [
        (LocalShaSource(False, b""), LocalClockSource(1700000000), LocalTargetSource("aarch64-apple-darwin")),
        (LocalShaSource(True, b"c3ff13cd\n"), LocalClockSource(None), LocalTargetSource("aarch64-apple-darwin")),
        (LocalShaSource(True, b"c3ff13cd\n"), LocalClockSource(1700000000), LocalTargetSource(None)),
        (LocalShaSource(False, b""), LocalClockSource(None), LocalTargetSource(None)),
    ]
    completed = 0
    for sha_src, clock_src, target_src in degraded_configurations:
        try:
            service = StampService(sha_src, clock_src, target_src, LocalPathProbe(frozenset()))
            service.plan(StampRequest("LUMEN", "../../.git/HEAD"))
            completed += 1
        except Exception:
            pass
    obs10 = completed
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. a_degraded_value_is_the_fallback_word_and_nothing_appended_to_it
    exp11 = BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[10][1]
    obs11 = plan.directives[0].value
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. the_design_exposes_no_error_or_result_type_from_the_entry_point
    exp12 = BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[11][1]
    own_classes = []
    for name, obj in inspect.getmembers(emit_stamp_mod, inspect.isclass):
        if obj.__module__ == emit_stamp_mod.__name__:
            own_classes.append(name)
    obs12 = tuple(sorted(own_classes))
    checks.append({
        "name": BEST_EFFORT_DEGRADATION_SECURITY_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    return {
        "case_id": "best-effort-degradation-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
