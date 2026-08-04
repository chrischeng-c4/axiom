from __future__ import annotations

from metrics_prometheus.application.accumulators import Counter, Gauge, Histogram, Latency
from metrics_prometheus.domain.bucket import Bucket
from metrics_prometheus.infrastructure.recording_cell import RecordingCellFactory

MINIMUM_CHECKS = 14

LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX = (
    ("counter_sequence_total", 7),
    ("counter_permutation_total", 7),
    ("gauge_last_value_written", 42),
    ("gauge_overwrite_smaller", 15),
    ("latency_durations_sum", 400),
    ("latency_durations_count", 3),
    ("histogram_bucket_counts_multiset", (2, 1, 1)),
    ("histogram_bucket_counts_permutation", (2, 1, 1)),
    ("histogram_permutation_sum", 565),
    ("histogram_permutation_count", 4),
    ("histogram_exact_bound_assignment", (1, 0, 0)),
    ("histogram_one_above_bound_assignment", (0, 1, 0)),
    ("histogram_one_per_bucket_batch", (1, 1, 1)),
    ("histogram_in_bounds_counts_sum_total", 3),
)


def verify_lock_free_accumulation_behavior() -> dict[str, object]:
    checks = []

    # 1. counter_sequence_total
    exp1 = LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[0][1]
    c1 = Counter(RecordingCellFactory(), "c1")
    c1.incr()
    c1.add(5)
    c1.incr()
    obs1 = c1.get()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1,
    })

    # 2. counter_permutation_total
    exp2 = LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[1][1]
    c2 = Counter(RecordingCellFactory(), "c2")
    c2.add(5)
    c2.incr()
    c2.incr()
    obs2 = c2.get()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. gauge_last_value_written
    exp3 = LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[2][1]
    g1 = Gauge(RecordingCellFactory(), "g1")
    g1.set(10)
    g1.set(42)
    obs3 = g1.get()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3,
    })

    # 4. gauge_overwrite_smaller
    exp4 = LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[3][1]
    g2 = Gauge(RecordingCellFactory(), "g2")
    g2.set(42)
    g2.set(15)
    obs4 = g2.get()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. latency_durations_sum
    exp5 = LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[4][1]
    lat1 = Latency(RecordingCellFactory(), "lat1")
    lat1.observe(100)
    lat1.observe(250)
    lat1.observe(50)
    obs5 = lat1.sum()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. latency_durations_count
    exp6 = LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[5][1]
    obs6 = lat1.count()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. histogram_bucket_counts_multiset
    exp7 = LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[6][1]
    bounds = (Bucket("10", 10), Bucket("100", 100), Bucket("1000", 1000))
    h1 = Histogram(RecordingCellFactory(), "h1", bounds)
    h1.observe(5)
    h1.observe(10)
    h1.observe(50)
    h1.observe(500)
    obs7 = h1.bucket_counts()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # 8. histogram_bucket_counts_permutation
    exp8 = LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[7][1]
    h2 = Histogram(RecordingCellFactory(), "h2", bounds)
    h2.observe(500)
    h2.observe(50)
    h2.observe(5)
    h2.observe(10)
    obs8 = h2.bucket_counts()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. histogram_permutation_sum
    exp9 = LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[8][1]
    obs9 = h2.sum()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    # 10. histogram_permutation_count
    exp10 = LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[9][1]
    obs10 = h2.count()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. histogram_exact_bound_assignment
    exp11 = LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[10][1]
    h3 = Histogram(RecordingCellFactory(), "h3", bounds)
    h3.observe(10)
    obs11 = h3.bucket_counts()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. histogram_one_above_bound_assignment
    exp12 = LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[11][1]
    h4 = Histogram(RecordingCellFactory(), "h4", bounds)
    h4.observe(11)
    obs12 = h4.bucket_counts()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    # 13. histogram_one_per_bucket_batch
    exp13 = LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[12][1]
    h5 = Histogram(RecordingCellFactory(), "h5", bounds)
    h5.observe(5)
    h5.observe(50)
    h5.observe(500)
    obs13 = h5.bucket_counts()
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[12][0],
        "expected": exp13,
        "observed": obs13,
        "passed": obs13 == exp13,
    })

    # 14. histogram_in_bounds_counts_sum_total
    exp14 = LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[13][1]
    obs14 = sum(h5.bucket_counts())
    checks.append({
        "name": LOCK_FREE_ACCUMULATION_BEHAVIOR_MATRIX[13][0],
        "expected": exp14,
        "observed": obs14,
        "passed": obs14 == exp14,
    })

    return {
        "case_id": "lock-free-accumulation-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
