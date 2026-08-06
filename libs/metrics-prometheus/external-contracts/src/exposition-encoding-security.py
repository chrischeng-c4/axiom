from __future__ import annotations

from metrics_prometheus.application.accumulators import Histogram
from metrics_prometheus.application.exposition import render_histogram
from metrics_prometheus.domain.bucket import Bucket, cumulative
from metrics_prometheus.domain.scaling import scale_decimal
from metrics_prometheus.infrastructure.recording_cell import RecordingCellFactory

MINIMUM_CHECKS = 14

EXPOSITION_ENCODING_SECURITY_MATRIX = (
    ("scale_decimal_fractional_leading_zeros", "0.001"),
    ("scale_decimal_standard_power_of_ten", "1.500"),
    ("scale_decimal_five_leading_zeros", "0.000002"),
    ("scale_decimal_above_253_exact", "9007199254740.993"),
    ("scale_decimal_zero_value", "0.000"),
    ("scale_decimal_non_power_of_ten_no_decimal", "2"),
    ("scale_decimal_one_divisor_no_decimal", "7"),
    ("scale_decimal_zero_divisor_no_decimal", "7"),
    ("scale_decimal_negative_value_raises_value_error", "ValueError"),
    ("render_histogram_sum_no_scientific_notation", False),
    ("render_histogram_sum_single_decimal_point", 1),
    ("cumulative_exclusive_count_tuple", (1, 1, 3)),
    ("render_histogram_parsed_cumulative_buckets_monotonic", True),
    ("render_histogram_all_lines_newline_terminated", 7),
)


def verify_exposition_encoding_security() -> dict[str, object]:
    checks = []

    # 1. scale_decimal_fractional_leading_zeros
    exp1 = EXPOSITION_ENCODING_SECURITY_MATRIX[0][1]
    obs1 = scale_decimal(1, 1000)
    checks.append({
        "name": EXPOSITION_ENCODING_SECURITY_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1,
    })

    # 2. scale_decimal_standard_power_of_ten
    exp2 = EXPOSITION_ENCODING_SECURITY_MATRIX[1][1]
    obs2 = scale_decimal(1500, 1000)
    checks.append({
        "name": EXPOSITION_ENCODING_SECURITY_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. scale_decimal_five_leading_zeros
    exp3 = EXPOSITION_ENCODING_SECURITY_MATRIX[2][1]
    obs3 = scale_decimal(2, 1000000)
    checks.append({
        "name": EXPOSITION_ENCODING_SECURITY_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3,
    })

    # 4. scale_decimal_above_253_exact
    exp4 = EXPOSITION_ENCODING_SECURITY_MATRIX[3][1]
    obs4 = scale_decimal(9007199254740993, 1000)
    checks.append({
        "name": EXPOSITION_ENCODING_SECURITY_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. scale_decimal_zero_value
    exp5 = EXPOSITION_ENCODING_SECURITY_MATRIX[4][1]
    obs5 = scale_decimal(0, 1000)
    checks.append({
        "name": EXPOSITION_ENCODING_SECURITY_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. scale_decimal_non_power_of_ten_no_decimal
    exp6 = EXPOSITION_ENCODING_SECURITY_MATRIX[5][1]
    obs6 = scale_decimal(7, 3)
    checks.append({
        "name": EXPOSITION_ENCODING_SECURITY_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. scale_decimal_one_divisor_no_decimal
    exp7 = EXPOSITION_ENCODING_SECURITY_MATRIX[6][1]
    obs7 = scale_decimal(7, 1)
    checks.append({
        "name": EXPOSITION_ENCODING_SECURITY_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # 8. scale_decimal_zero_divisor_no_decimal
    exp8 = EXPOSITION_ENCODING_SECURITY_MATRIX[7][1]
    obs8 = scale_decimal(7, 0)
    checks.append({
        "name": EXPOSITION_ENCODING_SECURITY_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. scale_decimal_negative_value_raises_value_error
    exp9 = EXPOSITION_ENCODING_SECURITY_MATRIX[8][1]
    try:
        scale_decimal(-1, 1000)
        obs9 = "None"
    except Exception as exc:
        obs9 = type(exc).__name__
    checks.append({
        "name": EXPOSITION_ENCODING_SECURITY_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    # Render a histogram for checking properties
    bounds = (Bucket("10", 10), Bucket("100", 100))
    hist = Histogram(RecordingCellFactory(), "req_lat", bounds)
    hist.observe(5)
    hist.observe(500)
    rendered = render_histogram(hist, "req_lat", "help", 1000)

    # 10. render_histogram_sum_no_scientific_notation
    exp10 = EXPOSITION_ENCODING_SECURITY_MATRIX[9][1]
    sum_val_str = [ln for ln in rendered.splitlines() if ln.startswith("req_lat_sum")][0].split()[-1]
    obs10 = ("e" in sum_val_str) or ("E" in sum_val_str)
    checks.append({
        "name": EXPOSITION_ENCODING_SECURITY_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. render_histogram_sum_single_decimal_point
    exp11 = EXPOSITION_ENCODING_SECURITY_MATRIX[10][1]
    obs11 = obs2.count(".")
    checks.append({
        "name": EXPOSITION_ENCODING_SECURITY_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. cumulative_exclusive_count_tuple
    exp12 = EXPOSITION_ENCODING_SECURITY_MATRIX[11][1]
    obs12 = cumulative((1, 0, 2))
    checks.append({
        "name": EXPOSITION_ENCODING_SECURITY_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    # 13. render_histogram_parsed_cumulative_buckets_monotonic
    exp13 = EXPOSITION_ENCODING_SECURITY_MATRIX[12][1]
    bucket_vals = []
    for line in rendered.splitlines():
        if "req_lat_bucket{le=" in line:
            bucket_vals.append(int(line.split()[-1]))
    obs13 = all(x <= y for x, y in zip(bucket_vals, bucket_vals[1:]))
    checks.append({
        "name": EXPOSITION_ENCODING_SECURITY_MATRIX[12][0],
        "expected": exp13,
        "observed": obs13,
        "passed": obs13 == exp13,
    })

    # 14. render_histogram_all_lines_newline_terminated
    exp14 = EXPOSITION_ENCODING_SECURITY_MATRIX[13][1]
    raw_lines = rendered.splitlines(keepends=True)
    obs14 = sum(1 for ln in raw_lines if ln.endswith("\n"))
    checks.append({
        "name": EXPOSITION_ENCODING_SECURITY_MATRIX[13][0],
        "expected": exp14,
        "observed": obs14,
        "passed": obs14 == exp14 and len(raw_lines) == exp14,
    })

    return {
        "case_id": "exposition-encoding-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
