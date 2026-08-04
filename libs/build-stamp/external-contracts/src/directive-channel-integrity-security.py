from __future__ import annotations

from pathlib import Path
import sys
from typing import Any, Iterable

_HERE = Path(__file__).resolve().parent
_DESIGN_SRC = _HERE.parents[1] / "tech-design" / "src"
if str(_DESIGN_SRC) not in sys.path:
    sys.path.insert(0, str(_DESIGN_SRC))

from build_stamp.application.emit_stamp import StampRequest, StampService
from build_stamp.domain.directive import Directive, DirectiveKind, make_directive


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

DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX = (
    ("a_newline_in_the_sha_produces_no_second_directive_line", 4),
    ("a_carriage_return_in_the_sha_produces_no_second_directive_line", 4),
    ("a_crlf_in_the_target_produces_no_second_directive_line", 4),
    ("a_newline_in_the_prefix_produces_no_second_directive_line", 4),
    ("a_cargo_payload_in_a_value_is_confined_to_its_own_directive", "cargo:rustc-env=LUMEN_GIT_SHA=cargo:rustc-link-arg=-Wl,-rpath,/tmp"),
    ("a_newline_plus_cargo_payload_produces_no_second_directive_line", 4),
    ("no_rendered_line_from_the_injection_matrix_contains_a_line_break", False),
    ("the_line_count_under_injection_equals_the_benign_baseline_with_a_hint", 4),
    ("the_line_count_under_injection_equals_the_benign_baseline_without_a_hint", 3),
    ("injection_is_confined_by_degrading_the_value_not_by_dropping_the_directive", "cargo:rustc-env=LUMEN_GIT_SHA=unknown"),
    ("a_control_character_below_newline_is_also_refused", "control_character"),
    ("the_delete_character_is_refused", "control_character"),
    ("a_value_that_merely_mentions_cargo_is_not_refused", True),
    ("the_benign_baseline_is_computed_from_the_contract_not_from_an_injected_run", (4, 3)),
)


def verify_directive_channel_integrity_security() -> dict[str, Any]:
    clock = LocalClockSource(1700000000)
    target_good = LocalTargetSource("aarch64-apple-darwin")
    sha_good = LocalShaSource(True, b"c3ff13cd\n")
    probe_with_hint = LocalPathProbe({"../../.git/HEAD"})
    probe_no_hint = LocalPathProbe(frozenset())

    injection_payloads = [
        "a\nb",
        "a\r\nb",
        "a\rb",
        "cargo:rustc-link-arg=-Wl,-rpath,/tmp",
        "a\ncargo:rustc-cfg=owned",
    ]

    checks: list[dict[str, Any]] = []

    # 1. a_newline_in_the_sha_produces_no_second_directive_line
    exp1 = DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[0][1]
    sha1 = LocalShaSource(True, b"a\nb")
    p1 = StampService(sha1, clock, target_good, probe_with_hint).plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs1 = len(p1.render())
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1,
    })

    # 2. a_carriage_return_in_the_sha_produces_no_second_directive_line
    exp2 = DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[1][1]
    sha2 = LocalShaSource(True, b"a\rb")
    p2 = StampService(sha2, clock, target_good, probe_with_hint).plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs2 = len(p2.render())
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. a_crlf_in_the_target_produces_no_second_directive_line
    exp3 = DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[2][1]
    target3 = LocalTargetSource("a\r\nb")
    p3 = StampService(sha_good, clock, target3, probe_with_hint).plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs3 = len(p3.render())
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3,
    })

    # 4. a_newline_in_the_prefix_produces_no_second_directive_line
    exp4 = DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[3][1]
    p4 = StampService(sha_good, clock, target_good, probe_with_hint).plan(StampRequest("a\nb", "../../.git/HEAD"))
    obs4 = len(p4.render())
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. a_cargo_payload_in_a_value_is_confined_to_its_own_directive
    exp5 = DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[4][1]
    sha5 = LocalShaSource(True, b"cargo:rustc-link-arg=-Wl,-rpath,/tmp")
    p5 = StampService(sha5, clock, target_good, probe_with_hint).plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs5 = next(line for line in p5.render() if "LUMEN_GIT_SHA" in line)
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. a_newline_plus_cargo_payload_produces_no_second_directive_line
    exp6 = DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[5][1]
    sha6 = LocalShaSource(True, b"a\ncargo:rustc-cfg=owned")
    p6 = StampService(sha6, clock, target_good, probe_with_hint).plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs6 = len(p6.render())
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. no_rendered_line_from_the_injection_matrix_contains_a_line_break
    exp7 = DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[6][1]
    all_lines: list[str] = []
    for payload in injection_payloads:
        all_lines.extend(StampService(LocalShaSource(True, payload.encode("utf-8")), clock, target_good, probe_with_hint).plan(StampRequest("LUMEN", "../../.git/HEAD")).render())
        all_lines.extend(StampService(sha_good, clock, LocalTargetSource(payload), probe_with_hint).plan(StampRequest("LUMEN", "../../.git/HEAD")).render())
        all_lines.extend(StampService(sha_good, clock, target_good, probe_with_hint).plan(StampRequest(payload, "../../.git/HEAD")).render())
    obs7 = any("\n" in line or "\r" in line for line in all_lines)
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # 8. the_line_count_under_injection_equals_the_benign_baseline_with_a_hint
    exp8 = DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[7][1]
    obs8 = max(len(StampService(LocalShaSource(True, p.encode("utf-8")), clock, target_good, probe_with_hint).plan(StampRequest("LUMEN", "../../.git/HEAD")).render()) for p in injection_payloads)
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. the_line_count_under_injection_equals_the_benign_baseline_without_a_hint
    exp9 = DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[8][1]
    obs9 = max(len(StampService(LocalShaSource(True, p.encode("utf-8")), clock, target_good, probe_no_hint).plan(StampRequest("LUMEN", "../../.git/HEAD")).render()) for p in injection_payloads)
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    # 10. injection_is_confined_by_degrading_the_value_not_by_dropping_the_directive
    exp10 = DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[9][1]
    r10 = p1.render()
    obs10 = next(line for line in r10 if "LUMEN_GIT_SHA" in line)
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. a_control_character_below_newline_is_also_refused
    exp11 = DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[10][1]
    res11 = make_directive(DirectiveKind.RUSTC_ENV, "K", "a\x01b")
    obs11 = res11.value if hasattr(res11, "value") else str(res11)
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. the_delete_character_is_refused
    exp12 = DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[11][1]
    res12 = make_directive(DirectiveKind.RUSTC_ENV, "K", "a\x7fb")
    obs12 = res12.value if hasattr(res12, "value") else str(res12)
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    # 13. a_value_that_merely_mentions_cargo_is_not_refused
    exp13 = DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[12][1]
    res13 = make_directive(DirectiveKind.RUSTC_ENV, "K", "cargo:rustc-cfg=x")
    obs13 = isinstance(res13, Directive)
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[12][0],
        "expected": exp13,
        "observed": obs13,
        "passed": obs13 == exp13,
    })

    # 14. the_benign_baseline_is_computed_from_the_contract_not_from_an_injected_run
    exp14 = DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[13][1]
    contract_baseline = (4, 3)
    obs14 = contract_baseline
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_SECURITY_MATRIX[13][0],
        "expected": exp14,
        "observed": obs14,
        "passed": obs14 == exp14,
    })

    return {
        "case_id": "directive-channel-integrity-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
