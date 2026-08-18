from __future__ import annotations

from service_observability.infrastructure.metrics import (
    LifecycleMetrics,
    Sample,
    render,
)

MINIMUM_CHECKS = 11

CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX = (
    ("a_fresh_set_of_counters_reads_zero_on_every_series", (0, 0, 0)),
    ("an_accepted_connection_increments_only_its_own_counter", (1, 0, 0)),
    ("a_rejected_connection_increments_only_its_own_counter", (0, 1, 0)),
    ("a_closed_connection_increments_only_its_own_counter", (0, 0, 1)),
    ("mixed_traffic_reads_back_the_exact_counts", (7, 2, 5)),
    ("the_three_canonical_series_appear_in_their_fixed_order", ('service_connections_accepted_total', 'service_connections_rejected_total', 'service_connections_closed_total')),
    ("every_sample_contributes_exactly_three_lines", (9, 0, 3)),
    ("each_series_carries_a_help_and_a_type_line_ahead_of_its_value", ('# HELP service_connections_accepted_total Total accepted service connections.', '# TYPE service_connections_accepted_total counter', 'service_connections_accepted_total 1')),
    ("the_counter_value_rendered_is_the_current_count", (('7', '2', '5'),)),
    ("the_exposition_ends_with_a_newline", (True, 9, True)),
    ("the_canonical_names_are_owned_by_this_crate", ('service_connections_accepted_total', 'service_connections_rejected_total', 'service_connections_closed_total')),
)


def metrics(accepted: int, rejected: int, closed: int) -> LifecycleMetrics:
    m = LifecycleMetrics()
    for _ in range(accepted):
        m.connection_accepted()
    for _ in range(rejected):
        m.connection_rejected()
    for _ in range(closed):
        m.connection_closed()
    return m


def verify_connection_lifecycle_metrics_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a fresh set of counters reads zero on every series
    exp1 = CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[0][1]
    fresh = LifecycleMetrics()
    obs1 = (fresh.accepted(), fresh.rejected(), fresh.closed())
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. an accepted connection increments only the accepted counter
    exp2 = CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[1][1]
    m1 = metrics(1, 0, 0)
    obs2 = (m1.accepted(), m1.rejected(), m1.closed())
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. a rejected connection increments only the rejected counter
    exp3 = CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[2][1]
    m2 = metrics(0, 1, 0)
    obs3 = (m2.accepted(), m2.rejected(), m2.closed())
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. a closed connection increments only the closed counter
    exp4 = CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[3][1]
    m3 = metrics(0, 0, 1)
    obs4 = (m3.accepted(), m3.rejected(), m3.closed())
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. mixed traffic reads back the exact counts
    exp5 = CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[4][1]
    mixed = metrics(7, 2, 5)
    obs5 = (mixed.accepted(), mixed.rejected(), mixed.closed())
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. the three canonical series appear in their fixed order
    exp6 = CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[5][1]
    body = metrics(1, 0, 0).render_metrics()
    obs6 = tuple((line.split()[0] for line in body.splitlines() if not line.startswith('#')))
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. every sample contributes exactly three lines
    exp7 = CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[6][1]
    obs7 = (len(body.splitlines()), len(render(()).splitlines()), len(render((Sample('one_total', 'counter', 'One.', 3),)).splitlines()))
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. each series carries a HELP and a TYPE line ahead of its value
    exp8 = CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[7][1]
    obs8 = tuple(body.splitlines()[:3])
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. the counter value rendered is the current count
    exp9 = CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[8][1]
    obs9 = (tuple((line.split()[1] for line in metrics(7, 2, 5).render_metrics().splitlines() if not line.startswith('#'))),)
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. the exposition ends with a newline so it concatenates cleanly
    exp10 = CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[9][1]
    obs10 = (body.endswith('\n'), body.count('\n'), render((Sample('a_total', 'counter', 'A.', 0),)).endswith('\n'))
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the canonical names are owned by this crate, not chosen per service
    exp11 = CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[10][1]
    obs11 = tuple((line.split()[2] for line in body.splitlines() if line.startswith('# HELP')))
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_BEHAVIOR_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "connection-lifecycle-metrics-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
