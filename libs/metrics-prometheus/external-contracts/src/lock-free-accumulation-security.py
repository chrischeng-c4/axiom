from __future__ import annotations

from metrics_prometheus.application.accumulators import Counter, Histogram, Latency
from metrics_prometheus.domain.bucket import Bucket, assign
from metrics_prometheus.domain.scaling import scale_decimal
from metrics_prometheus.infrastructure.recording_cell import RecordingCellFactory

MINIMUM_CHECKS = 14

LOCK_FREE_ACCUMULATION_SECURITY_MATRIX = (
    ("histogram_overflow_leaves_buckets_unchanged", (0, 0)),
    ("histogram_overflow_raises_count", 1),
    ("histogram_overflow_raises_sum", 5000),
    ("histogram_bucket_sum_equals_total_minus_overflow", 2),
    ("histogram_second_overflow_preserves_buckets", ((0, 0), 2, 11000)),
    ("assign_above_all_bounds_none", None),
    ("assign_lowest_bound_index", 0),
    ("counter_negative_delta_raises_value_error", "ValueError"),
    ("counter_value_after_rejected_add_unchanged", 0),
    ("latency_negative_duration_raises_value_error", "ValueError"),
    ("counter_monotonic_readings_sequence", (0, 1, 6, 7)),
    ("sum_above_253_exact_integer", 9007199254741993),
    ("scale_decimal_large_sum_power_of_ten", "9007199254741.993"),
    ("histogram_observe_cell_op_order", (("hsnap_bucket_0", "add"), ("hsnap_sum", "add"), ("hsnap_count", "add"))),
)


def verify_lock_free_accumulation_security() -> dict[str, object]:
    checks = []

    bounds = (Bucket("10", 10), Bucket("100", 100))
    f1 = RecordingCellFactory()
    h1 = Histogram(f1, "h1", bounds)
    bucket_before = h1.bucket_counts()
    h1.observe(5000)

    # 1. histogram_overflow_leaves_buckets_unchanged
    exp1 = LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[0][1]
    obs1 = h1.bucket_counts()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1 and bucket_before == exp1,
    })

    # 2. histogram_overflow_raises_count
    exp2 = LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[1][1]
    obs2 = h1.count()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. histogram_overflow_raises_sum
    exp3 = LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[2][1]
    obs3 = h1.sum()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3,
    })

    # 4. histogram_bucket_sum_equals_total_minus_overflow
    exp4 = LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[3][1]
    h_mix = Histogram(RecordingCellFactory(), "hmix", bounds)
    h_mix.observe(5)
    h_mix.observe(50)
    h_mix.observe(5000)
    obs4 = sum(h_mix.bucket_counts())
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. histogram_second_overflow_preserves_buckets
    exp5 = LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[4][1]
    h1.observe(6000)
    obs5 = (h1.bucket_counts(), h1.count(), h1.sum())
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. assign_above_all_bounds_none
    exp6 = LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[5][1]
    obs6 = assign(bounds, 500)
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. assign_lowest_bound_index
    exp7 = LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[6][1]
    obs7 = assign(bounds, 10)
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # 8. counter_negative_delta_raises_value_error
    exp8 = LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[7][1]
    c1 = Counter(RecordingCellFactory(), "c1")
    try:
        c1.add(-1)
        obs8 = "None"
    except Exception as exc:
        obs8 = type(exc).__name__
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. counter_value_after_rejected_add_unchanged
    exp9 = LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[8][1]
    obs9 = c1.get()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    # 10. latency_negative_duration_raises_value_error
    exp10 = LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[9][1]
    lat = Latency(RecordingCellFactory(), "lat")
    try:
        lat.observe(-5)
        obs10 = "None"
    except Exception as exc:
        obs10 = type(exc).__name__
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. counter_monotonic_readings_sequence
    exp11 = LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[10][1]
    c2 = Counter(RecordingCellFactory(), "c2")
    r0 = c2.get()
    c2.incr()
    r1 = c2.get()
    c2.add(5)
    r2 = c2.get()
    c2.incr()
    r3 = c2.get()
    obs11 = (r0, r1, r2, r3)
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. sum_above_253_exact_integer
    exp12 = LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[11][1]
    c_large = Counter(RecordingCellFactory(), "clarge")
    c_large.add(9007199254740993)
    c_large.add(1000)
    obs12 = c_large.get()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    # 13. scale_decimal_large_sum_power_of_ten
    exp13 = LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[12][1]
    obs13 = scale_decimal(obs12, 1000)
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[12][0],
        "expected": exp13,
        "observed": obs13,
        "passed": obs13 == exp13,
    })

    # 14. histogram_observe_cell_op_order
    exp14 = LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[13][1]
    f_snap = RecordingCellFactory()
    h_snap = Histogram(f_snap, "hsnap", bounds)
    h_snap.observe(5)
    log_snap = list(f_snap.log)
    obs14 = tuple((cell_name, op) for cell_name, op in log_snap)
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_SECURITY_MATRIX[13][0],
        "expected": exp14,
        "observed": obs14,
        "passed": obs14 == exp14,
    })

    return {
        "case_id": "lock-free-accumulation-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
