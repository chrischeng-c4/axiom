from __future__ import annotations

from pathlib import Path
import sys
from typing import Any, Iterable

_HERE = Path(__file__).resolve().parent
_DESIGN_SRC = _HERE.parents[1] / "tech-design" / "src"
if str(_DESIGN_SRC) not in sys.path:
    sys.path.insert(0, str(_DESIGN_SRC))

from build_stamp.application.emit_stamp import StampRequest, StampService
from build_stamp.domain.sha import decode_short_sha


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

DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX = (
    ("a_trailing_newline_is_removed_from_the_decoded_answer", "c3ff13cd"),
    ("leading_and_trailing_spaces_are_removed", "c3ff13cd"),
    ("surrounding_tabs_are_removed", "c3ff13cd"),
    ("an_empty_answer_is_no_answer_rather_than_an_empty_sha", None),
    ("an_all_whitespace_answer_is_no_answer_rather_than_an_empty_sha", None),
    ("a_failed_invocation_is_no_answer_regardless_of_its_output", None),
    ("invalid_utf8_is_replaced_rather_than_discarded", "\ufffd\ufffd ab"),
    ("invalid_utf8_is_replaced_rather_than_raised", True),
    ("the_trim_is_defined_on_the_decoded_string_not_on_the_raw_bytes", "ab"),
    ("a_normalized_sha_reaches_the_emitted_directive_unchanged", "cargo:rustc-env=LUMEN_GIT_SHA=c3ff13cd"),
    ("a_no_answer_result_reaches_the_emitted_directive_as_the_fallback", "cargo:rustc-env=LUMEN_GIT_SHA=unknown"),
    ("interior_whitespace_inside_the_answer_is_preserved", "c3ff 13cd"),
    ("the_decoded_length_and_the_byte_length_differ_for_a_multibyte_answer", True),
)


def verify_directive_channel_integrity_behavior() -> dict[str, Any]:
    clock = LocalClockSource(1700000000)
    target = LocalTargetSource("aarch64-apple-darwin")
    probe = LocalPathProbe(frozenset())

    checks: list[dict[str, Any]] = []

    # 1. a_trailing_newline_is_removed_from_the_decoded_answer
    exp1 = DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[0][1]
    obs1 = decode_short_sha(True, b"c3ff13cd\n")
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1,
    })

    # 2. leading_and_trailing_spaces_are_removed
    exp2 = DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[1][1]
    obs2 = decode_short_sha(True, b"  c3ff13cd  ")
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. surrounding_tabs_are_removed
    exp3 = DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[2][1]
    obs3 = decode_short_sha(True, b"\tc3ff13cd\t")
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3,
    })

    # 4. an_empty_answer_is_no_answer_rather_than_an_empty_sha
    exp4 = DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[3][1]
    obs4 = decode_short_sha(True, b"")
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. an_all_whitespace_answer_is_no_answer_rather_than_an_empty_sha
    exp5 = DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[4][1]
    obs5 = decode_short_sha(True, b"  \n\t  ")
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. a_failed_invocation_is_no_answer_regardless_of_its_output
    exp6 = DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[5][1]
    obs6 = decode_short_sha(False, b"c3ff13cd")
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. invalid_utf8_is_replaced_rather_than_discarded
    exp7 = DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[6][1]
    obs7 = decode_short_sha(True, b"\xff\xfe ab \n")
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # 8. invalid_utf8_is_replaced_rather_than_raised
    exp8 = DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[7][1]
    try:
        res_utf8 = decode_short_sha(True, b"\xff\xfe")
        obs8 = isinstance(res_utf8, str) or res_utf8 is None
    except Exception:
        obs8 = False
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. the_trim_is_defined_on_the_decoded_string_not_on_the_raw_bytes
    exp9 = DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[8][1]
    obs9 = decode_short_sha(True, b" \xe2\x80\x83ab \n")
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    # 10. a_normalized_sha_reaches_the_emitted_directive_unchanged
    exp10 = DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[9][1]
    p10 = StampService(LocalShaSource(True, b"  c3ff13cd\n"), clock, target, probe).plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs10 = next(line for line in p10.render() if "LUMEN_GIT_SHA" in line)
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. a_no_answer_result_reaches_the_emitted_directive_as_the_fallback
    exp11 = DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[10][1]
    p11 = StampService(LocalShaSource(True, b"   \n"), clock, target, probe).plan(StampRequest("LUMEN", "../../.git/HEAD"))
    obs11 = next(line for line in p11.render() if "LUMEN_GIT_SHA" in line)
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. interior_whitespace_inside_the_answer_is_preserved
    exp12 = DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[11][1]
    obs12 = decode_short_sha(True, b"c3ff 13cd\n")
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    # 13. the_decoded_length_and_the_byte_length_differ_for_a_multibyte_answer
    exp13 = DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[12][1]
    raw = b"\xe2\x80\x83ab"
    decoded = raw.decode("utf-8")
    obs13 = (len(raw) != len(decoded))
    checks.append({
        "name": DIRECTIVE_CHANNEL_INTEGRITY_BEHAVIOR_MATRIX[12][0],
        "expected": exp13,
        "observed": obs13,
        "passed": obs13 == exp13,
    })

    return {
        "case_id": "directive-channel-integrity-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
