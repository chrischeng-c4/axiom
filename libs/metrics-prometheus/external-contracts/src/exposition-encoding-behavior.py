from __future__ import annotations

from metrics_prometheus.application.accumulators import Histogram
from metrics_prometheus.application.exposition import render, render_histogram, render_labeled
from metrics_prometheus.domain.bucket import Bucket
from metrics_prometheus.domain.sample import Label, LabeledSample, MetricKind, Sample, SampleGroup
from metrics_prometheus.infrastructure.recording_cell import RecordingCellFactory

MINIMUM_CHECKS = 14

EXPOSITION_ENCODING_BEHAVIOR_MATRIX = (
    ("render_single_counter_sample", "# HELP n h\n# TYPE n counter\nn 3\n"),
    ("render_two_families_caller_order", "# HELP n h\n# TYPE n counter\nn 3\n# HELP g h2\n# TYPE g gauge\ng 42\n"),
    ("render_two_families_reversed_order", "# HELP g h2\n# TYPE g gauge\ng 42\n# HELP n h\n# TYPE n counter\nn 3\n"),
    ("render_family_help_precedes_type", ("# HELP n h", "# TYPE n counter")),
    ("render_type_counter_kind_word", "counter"),
    ("render_type_gauge_kind_word", "gauge"),
    ("render_deterministic_byte_equality", True),
    ("render_labeled_group_line_count", 4),
    ("render_histogram_cumulative_bucket_rows", (1, 1, 3)),
    ("render_histogram_inf_row_total_count", 3),
    ("render_histogram_inf_exceeds_last_finite_with_overflow", (1, 3)),
    ("render_histogram_sum_line_literal", "req_latency_sum 0.505\n"),
    ("render_histogram_count_line_literal", "req_latency_count 2\n"),
    ("render_histogram_full_multiline_output", "# HELP req_latency Request latency in seconds\n# TYPE req_latency histogram\nreq_latency_bucket{le=\"10\"} 1\nreq_latency_bucket{le=\"100\"} 1\nreq_latency_bucket{le=\"+Inf\"} 2\nreq_latency_sum 0.505\nreq_latency_count 2\n"),
)


def verify_exposition_encoding_behavior() -> dict[str, object]:
    checks = []

    sample_counter = Sample(name="n", kind=MetricKind.COUNTER, help="h", value=3)
    sample_gauge = Sample(name="g", kind=MetricKind.GAUGE, help="h2", value=42)

    # 1. render_single_counter_sample
    exp1 = EXPOSITION_ENCODING_BEHAVIOR_MATRIX[0][1]
    obs1 = render((sample_counter,))
    checks.append({
        "name": EXPOSITION_ENCODING_BEHAVIOR_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1,
    })

    # 2. render_two_families_caller_order
    exp2 = EXPOSITION_ENCODING_BEHAVIOR_MATRIX[1][1]
    obs2 = render((sample_counter, sample_gauge))
    checks.append({
        "name": EXPOSITION_ENCODING_BEHAVIOR_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. render_two_families_reversed_order
    exp3 = EXPOSITION_ENCODING_BEHAVIOR_MATRIX[2][1]
    obs3 = render((sample_gauge, sample_counter))
    checks.append({
        "name": EXPOSITION_ENCODING_BEHAVIOR_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3,
    })

    # 4. render_family_help_precedes_type
    exp4 = EXPOSITION_ENCODING_BEHAVIOR_MATRIX[3][1]
    lines1 = obs1.strip().split("\n")
    obs4 = (lines1[0], lines1[1])
    checks.append({
        "name": EXPOSITION_ENCODING_BEHAVIOR_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. render_type_counter_kind_word
    exp5 = EXPOSITION_ENCODING_BEHAVIOR_MATRIX[4][1]
    obs5 = lines1[1].split()[-1]
    checks.append({
        "name": EXPOSITION_ENCODING_BEHAVIOR_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. render_type_gauge_kind_word
    exp6 = EXPOSITION_ENCODING_BEHAVIOR_MATRIX[5][1]
    obs_gauge_text = render((sample_gauge,))
    obs6 = obs_gauge_text.strip().split("\n")[1].split()[-1]
    checks.append({
        "name": EXPOSITION_ENCODING_BEHAVIOR_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. render_deterministic_byte_equality
    exp7 = EXPOSITION_ENCODING_BEHAVIOR_MATRIX[6][1]
    r1 = render((sample_counter,))
    r2 = render((sample_counter,))
    obs7 = (r1 == r2) and (r1 == exp1)
    checks.append({
        "name": EXPOSITION_ENCODING_BEHAVIOR_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # 8. render_labeled_group_line_count
    exp8 = EXPOSITION_ENCODING_BEHAVIOR_MATRIX[7][1]
    group = SampleGroup(
        name="http_requests",
        kind=MetricKind.COUNTER,
        help="h",
        samples=(
            LabeledSample(labels=(Label("path", "/a"),), value=1),
            LabeledSample(labels=(Label("path", "/b"),), value=2),
        ),
    )
    rendered_group = render_labeled((group,))
    obs8 = len([ln for ln in rendered_group.split("\n") if ln])
    checks.append({
        "name": EXPOSITION_ENCODING_BEHAVIOR_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. render_histogram_cumulative_bucket_rows
    exp9 = EXPOSITION_ENCODING_BEHAVIOR_MATRIX[8][1]
    bounds3 = (Bucket("10", 10), Bucket("100", 100), Bucket("1000", 1000))
    hist3 = Histogram(RecordingCellFactory(), "h3", bounds3)
    hist3.observe(5)
    hist3.observe(200)
    hist3.observe(300)
    rendered_h3 = render_histogram(hist3, "h3", "help", 1)
    h3_lines = rendered_h3.strip().split("\n")
    bucket_vals = []
    for line in h3_lines:
        if "h3_bucket{le=" in line and 'le="+Inf"' not in line:
            bucket_vals.append(int(line.split()[-1]))
    obs9 = tuple(bucket_vals)
    checks.append({
        "name": EXPOSITION_ENCODING_BEHAVIOR_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    # 10. render_histogram_inf_row_total_count
    exp10 = EXPOSITION_ENCODING_BEHAVIOR_MATRIX[9][1]
    inf_val = None
    for line in h3_lines:
        if 'le="+Inf"' in line:
            inf_val = int(line.split()[-1])
    obs10 = inf_val
    checks.append({
        "name": EXPOSITION_ENCODING_BEHAVIOR_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. render_histogram_inf_exceeds_last_finite_with_overflow
    exp11 = EXPOSITION_ENCODING_BEHAVIOR_MATRIX[10][1]
    bounds_ov = (Bucket("10", 10), Bucket("100", 100))
    hist_ov = Histogram(RecordingCellFactory(), "hov", bounds_ov)
    hist_ov.observe(5)
    hist_ov.observe(500)
    hist_ov.observe(600)
    rendered_ov = render_histogram(hist_ov, "hov", "help", 1)
    ov_lines = rendered_ov.strip().split("\n")
    last_finite = None
    inf_row = None
    for line in ov_lines:
        if 'hov_bucket{le="100"}' in line:
            last_finite = int(line.split()[-1])
        elif 'hov_bucket{le="+Inf"}' in line:
            inf_row = int(line.split()[-1])
    obs11 = (last_finite, inf_row)
    checks.append({
        "name": EXPOSITION_ENCODING_BEHAVIOR_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. render_histogram_sum_line_literal
    exp12 = EXPOSITION_ENCODING_BEHAVIOR_MATRIX[11][1]
    bounds2 = (Bucket("10", 10), Bucket("100", 100))
    hist2 = Histogram(RecordingCellFactory(), "req_latency", bounds2)
    hist2.observe(5)
    hist2.observe(500)
    rendered_h2 = render_histogram(hist2, "req_latency", "Request latency in seconds", 1000)
    h2_lines = rendered_h2.splitlines(keepends=True)
    obs12 = [ln for ln in h2_lines if ln.startswith("req_latency_sum")][0]
    checks.append({
        "name": EXPOSITION_ENCODING_BEHAVIOR_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    # 13. render_histogram_count_line_literal
    exp13 = EXPOSITION_ENCODING_BEHAVIOR_MATRIX[12][1]
    obs13 = [ln for ln in h2_lines if ln.startswith("req_latency_count")][0]
    checks.append({
        "name": EXPOSITION_ENCODING_BEHAVIOR_MATRIX[12][0],
        "expected": exp13,
        "observed": obs13,
        "passed": obs13 == exp13,
    })

    # 14. render_histogram_full_multiline_output
    exp14 = EXPOSITION_ENCODING_BEHAVIOR_MATRIX[13][1]
    obs14 = rendered_h2
    checks.append({
        "name": EXPOSITION_ENCODING_BEHAVIOR_MATRIX[13][0],
        "expected": exp14,
        "observed": obs14,
        "passed": obs14 == exp14,
    })

    return {
        "case_id": "exposition-encoding-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
