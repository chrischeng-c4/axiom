from __future__ import annotations

from service_observability.infrastructure.process import (
    RSS_KIB_TO_BYTES,
    U64_MAX,
    parse_cpu_time,
    parse_ps_usage,
    parse_rss_kib,
)

MINIMUM_CHECKS = 14

PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX = (
    ("output_with_no_fields_at_all_is_an_error", ('ProcessSampleError', 'ProcessSampleError')),
    ("a_non_numeric_resident_size_is_an_error", ('ProcessSampleError', 'ProcessSampleError', 'ProcessSampleError')),
    ("a_missing_cpu_time_field_is_an_error_rather_than_a_zero", ('ProcessSampleError', 'ProcessSampleError')),
    ("a_non_numeric_clock_field_is_an_error", ('ProcessSampleError', 'ProcessSampleError', 'ProcessSampleError')),
    ("an_unrecognized_clock_shape_is_an_error", ('ProcessSampleError', 'ProcessSampleError', 'ProcessSampleError')),
    ("a_non_numeric_day_prefix_is_an_error", ('ProcessSampleError', 'ProcessSampleError', 'ProcessSampleError')),
    ("every_refusal_is_one_named_error_type", ('ProcessSampleError', 'ProcessSampleError', 'ProcessSampleError')),
    ("the_sampling_refusal_is_a_value_error", (True, True)),
    ("a_resident_reading_past_the_representable_range_is_refused", (18446744073709551615, 'ProcessSampleError', 'ProcessSampleError')),
    ("inside_the_range_the_byte_count_saturates_rather_than_wrapping", (18446744073709551615, 18446744073709551615, True)),
    ("the_saturation_boundary_is_exact_on_the_low_side", (18446744073709550592, True)),
    ("a_negative_resident_reading_is_refused", ('ProcessSampleError', 'ProcessSampleError', 'ProcessSampleError')),
    ("only_a_plain_ascii_decimal_is_a_resident_size", (1024, 'ProcessSampleError', 'ProcessSampleError', 'ProcessSampleError')),
    ("a_valid_reading_still_parses_after_all_the_refusals", (1048576, 1.0)),
)


def refusal(fn, *args, **kwargs) -> str:
    """The exception type a refusing call raises, or 'accepted' if it returns."""
    try:
        fn(*args, **kwargs)
    except Exception as exc:  # noqa: BLE001 — the type is the observation
        return type(exc).__name__
    return "accepted"


def _capture(fn, *args):
    try:
        fn(*args)
    except Exception as exc:  # noqa: BLE001
        return exc
    return None


def verify_portable_process_sampling_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. output with no fields at all is an error, not a zero sample
    exp1 = PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[0][1]
    obs1 = (refusal(parse_ps_usage, ''), refusal(parse_ps_usage, '   '))
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a non-numeric resident size is an error
    exp2 = PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[1][1]
    obs2 = (refusal(parse_ps_usage, 'abc 00:01.00'), refusal(parse_ps_usage, '1.5 00:01.00'), refusal(parse_ps_usage, '- 00:01.00'))
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a missing CPU-time field is an error rather than a zero
    exp3 = PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[2][1]
    obs3 = (refusal(parse_ps_usage, '1024'), refusal(parse_ps_usage, '  1024  '))
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a non-numeric clock field is an error
    exp4 = PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[3][1]
    obs4 = (refusal(parse_cpu_time, 'aa:bb'), refusal(parse_cpu_time, '01:xx'), refusal(parse_ps_usage, '1024 aa:bb'))
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. an unrecognized clock shape is an error, not a shape it guesses at
    exp5 = PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[4][1]
    obs5 = (refusal(parse_cpu_time, '90'), refusal(parse_cpu_time, '01:02:03:04'), refusal(parse_cpu_time, ''))
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. a non-numeric day prefix is an error
    exp6 = PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[5][1]
    obs6 = (refusal(parse_cpu_time, 'x-01:00:00'), refusal(parse_cpu_time, '-01:00:00'), refusal(parse_cpu_time, '1-2-01:00:00'))
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. every refusal is one named error type
    exp7 = PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[6][1]
    obs7 = (refusal(parse_cpu_time, '90'), refusal(parse_ps_usage, ''), refusal(parse_ps_usage, 'abc 1'))
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. the refusal is a ValueError, so an existing handler still catches it
    exp8 = PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[7][1]
    obs8 = (isinstance(_capture(parse_cpu_time, '90'), ValueError), isinstance(_capture(parse_ps_usage, ''), ValueError))
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a resident reading is refused past the representable range
    exp9 = PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[8][1]
    obs9 = (U64_MAX, refusal(parse_ps_usage, f'{U64_MAX + 1} 00:00.00'), refusal(parse_ps_usage, '99999999999999999999999999 00:00.00'))
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. inside the range the byte count saturates rather than wrapping
    exp10 = PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[9][1]
    saturating = -(-U64_MAX // RSS_KIB_TO_BYTES)
    obs10 = (parse_ps_usage(f'{U64_MAX} 00:00.00').rss_bytes, parse_ps_usage(f'{saturating} 00:00.00').rss_bytes, parse_ps_usage(f'{saturating} 00:00.00').rss_bytes == U64_MAX)
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the saturation boundary is exact on the low side
    exp11 = PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[10][1]
    just_under = saturating - 1
    obs11 = (parse_ps_usage(f'{just_under} 00:00.00').rss_bytes, parse_ps_usage(f'{just_under} 00:00.00').rss_bytes < U64_MAX)
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    # 12. a negative resident reading is refused, never read as a size
    exp12 = PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[11][1]
    obs12 = (refusal(parse_ps_usage, '-1 00:00.00'), refusal(parse_ps_usage, '-0 00:00.00'), refusal(parse_rss_kib, '-1024'))
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[11][0], "expected": exp12,
                   "observed": obs12, "passed": obs12 == exp12})

    # 13. only a plain ASCII decimal is a resident size
    exp13 = PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[12][1]
    obs13 = (parse_rss_kib('+1024'), refusal(parse_rss_kib, '1_024'), refusal(parse_rss_kib, '１０'), refusal(parse_rss_kib, '²'))
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[12][0], "expected": exp13,
                   "observed": obs13, "passed": obs13 == exp13})

    # 14. a valid reading still parses after all the refusals above
    exp14 = PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[13][1]
    obs14 = (parse_ps_usage('1024 00:01.00').rss_bytes, parse_ps_usage('1024 00:01.00').cpu_seconds)
    checks.append({"name": PORTABLE_PROCESS_SAMPLING_SECURITY_MATRIX[13][0], "expected": exp14,
                   "observed": obs14, "passed": obs14 == exp14})

    return {
        "case_id": "portable-process-sampling-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
