from __future__ import annotations

from service_observability.infrastructure.process import (
    RSS_KIB_TO_BYTES,
    parse_cpu_time,
    parse_ps_usage,
)

MINIMUM_CHECKS = 10

PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX = (
    ("the_two_field_clock_shape_converts_to_exact_seconds", (90.5, 0.0, 600.0)),
    ("the_three_field_clock_shape_converts_to_exact_seconds", (7384.0, 1.0, 360000.0)),
    ("the_day_prefixed_shape_adds_whole_days_to_the_clock", (86400.0, 93784.0, 259201.0)),
    ("a_fractional_second_survives_the_conversion", (0.25, 60.5, 86400.75)),
    ("a_resident_size_in_kib_is_published_in_bytes", (1024, 1048576, 0)),
    ("one_sample_carries_both_readings_together", (90.5, 2097152)),
    ("whitespace_between_the_fields_is_tolerated", (524288, 7384.0)),
    ("extra_trailing_fields_beyond_the_two_read_are_ignored", (102400, 1.0)),
    ("the_sample_is_a_plain_value_so_identical_readings_compare_equal", (True, False)),
    ("a_long_soak_reading_keeps_its_day_component", (1231200.0, 1231200.0, 8388608)),
)


def verify_portable_process_sampling_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the two-field clock shape converts to exact seconds
    exp1 = PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[0][1]
    obs1 = (parse_cpu_time('01:30.50'), parse_cpu_time('00:00.00'), parse_cpu_time('10:00'))
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. the three-field clock shape converts to exact seconds
    exp2 = PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[1][1]
    obs2 = (parse_cpu_time('02:03:04'), parse_cpu_time('00:00:01'), parse_cpu_time('100:00:00'))
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the day-prefixed shape adds whole days to the clock
    exp3 = PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[2][1]
    obs3 = (parse_cpu_time('1-00:00:00'), parse_cpu_time('1-02:03:04'), parse_cpu_time('3-00:00:01'))
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a fractional second survives the conversion
    exp4 = PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[3][1]
    obs4 = (parse_cpu_time('00:00.25'), parse_cpu_time('00:01:00.50'), parse_cpu_time('1-00:00:00.75'))
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. a resident size in KiB is published in bytes
    exp5 = PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[4][1]
    obs5 = (RSS_KIB_TO_BYTES, parse_ps_usage('1024 00:00.00').rss_bytes, parse_ps_usage('0 00:00.00').rss_bytes)
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. one sample carries both readings together
    exp6 = PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[5][1]
    sample = parse_ps_usage("2048 01:30.50")
    obs6 = (sample.cpu_seconds, sample.rss_bytes)
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. trailing and repeated whitespace between the fields is tolerated
    exp7 = PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[6][1]
    obs7 = (parse_ps_usage('  512   02:03:04  ').rss_bytes, parse_ps_usage('  512   02:03:04  ').cpu_seconds)
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. extra trailing fields beyond the two read are ignored
    exp8 = PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[7][1]
    obs8 = (parse_ps_usage('100 00:01.00 extra ignored').rss_bytes, parse_ps_usage('100 00:01.00 extra ignored').cpu_seconds)
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the sample is a plain value, so two identical readings compare equal
    exp9 = PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[8][1]
    obs9 = (parse_ps_usage('64 00:02.00') == parse_ps_usage('64 00:02.00'), parse_ps_usage('64 00:02.00') == parse_ps_usage('64 00:03.00'))
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. a long soak reading converts without losing the day component
    exp10 = PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[9][1]
    obs10 = (parse_cpu_time('14-06:00:00'), parse_ps_usage('8192 14-06:00:00').cpu_seconds, parse_ps_usage('8192 14-06:00:00').rss_bytes)
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    return {
        "case_id": "portable-process-sampling-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
