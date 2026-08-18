from __future__ import annotations

from metrics_prometheus.application.exposition import render_label_set, render_labeled
from metrics_prometheus.domain.escaping import escape_label_value
from metrics_prometheus.domain.label_order import canonical
from metrics_prometheus.domain.sample import Label, LabeledSample, MetricKind, SampleGroup

MINIMUM_CHECKS = 14

LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX = (
    ("render_label_set_first_order", '{a="2",b="1"}'),
    ("render_label_set_opposite_order", '{a="2",b="1"}'),
    ("render_label_set_order_equality", True),
    ("render_label_set_empty_tuple", ""),
    ("render_label_set_empty_no_braces", 0),
    ("canonical_label_tuple_sorted_by_name", (("a", "2"), ("z", "1"))),
    ("canonical_label_tuple_sorted_by_value_on_same_name", (("a", "1"), ("a", "2"))),
    ("canonical_label_tuple_idempotent", (("a", "1"), ("a", "2"))),
    ("escape_label_value_ordinary_string_unchanged", "plain"),
    ("render_label_set_single_label_formatting", '{env="prod"}'),
    ("render_labeled_sample_group_separate_lines", "# HELP http_requests h\n# TYPE http_requests counter\nhttp_requests{env=\"prod\"} 1\nhttp_requests{env=\"staging\"} 2\n"),
    ("render_labeled_unlabeled_sample_no_braces", "http_requests 10"),
    ("render_label_set_comma_separator_no_spaces", 0),
    ("render_label_set_separator_count", 2),
)


def verify_label_value_containment_behavior() -> dict[str, object]:
    checks = []

    l_a2 = Label("a", "2")
    l_b1 = Label("b", "1")

    # 1. render_label_set_first_order
    exp1 = LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[0][1]
    obs1 = render_label_set((l_b1, l_a2))
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1,
    })

    # 2. render_label_set_opposite_order
    exp2 = LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[1][1]
    obs2 = render_label_set((l_a2, l_b1))
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. render_label_set_order_equality
    exp3 = LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[2][1]
    obs3 = (obs1 == obs2)
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3,
    })

    # 4. render_label_set_empty_tuple
    exp4 = LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[3][1]
    obs4 = render_label_set(())
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. render_label_set_empty_no_braces
    exp5 = LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[4][1]
    obs5 = obs4.count("{")
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. canonical_label_tuple_sorted_by_name
    exp6 = LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[5][1]
    c_res1 = canonical((Label("z", "1"), Label("a", "2")))
    obs6 = tuple((l.name, l.value) for l in c_res1)
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. canonical_label_tuple_sorted_by_value_on_same_name
    exp7 = LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[6][1]
    c_res2 = canonical((Label("a", "2"), Label("a", "1")))
    obs7 = tuple((l.name, l.value) for l in c_res2)
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # 8. canonical_label_tuple_idempotent
    exp8 = LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[7][1]
    c_res3 = canonical(c_res2)
    obs8 = tuple((l.name, l.value) for l in c_res3)
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. escape_label_value_ordinary_string_unchanged
    exp9 = LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[8][1]
    obs9 = escape_label_value("plain")
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    # 10. render_label_set_single_label_formatting
    exp10 = LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[9][1]
    obs10 = render_label_set((Label("env", "prod"),))
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. render_labeled_sample_group_separate_lines
    exp11 = LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[10][1]
    group = SampleGroup(
        name="http_requests",
        kind=MetricKind.COUNTER,
        help="h",
        samples=(
            LabeledSample(labels=(Label("env", "prod"),), value=1),
            LabeledSample(labels=(Label("env", "staging"),), value=2),
        ),
    )
    obs11 = render_labeled((group,))
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. render_labeled_unlabeled_sample_no_braces
    exp12 = LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[11][1]
    group_unlabeled = SampleGroup(
        name="http_requests",
        kind=MetricKind.COUNTER,
        help="h",
        samples=(
            LabeledSample(labels=(), value=10),
        ),
    )
    rendered_unlbl = render_labeled((group_unlabeled,))
    obs12 = rendered_unlbl.splitlines()[-1]
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    # 13. render_label_set_comma_separator_no_spaces
    exp13 = LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[12][1]
    three_labels = (Label("a", "1"), Label("b", "2"), Label("c", "3"))
    rendered_3 = render_label_set(three_labels)
    obs13 = rendered_3.count(", ")
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[12][0],
        "expected": exp13,
        "observed": obs13,
        "passed": obs13 == exp13,
    })

    # 14. render_label_set_separator_count
    exp14 = LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[13][1]
    obs14 = rendered_3.count(",")
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_BEHAVIOR_MATRIX[13][0],
        "expected": exp14,
        "observed": obs14,
        "passed": obs14 == exp14,
    })

    return {
        "case_id": "label-value-containment-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
