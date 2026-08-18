from __future__ import annotations

from service_observability.infrastructure.metrics import (
    LifecycleMetrics,
    Sample,
    render,
)

MINIMUM_CHECKS = 11

CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX = (
    ("a_provider_with_no_samples_renders_an_empty_body", ('', True, 'accepted')),
    ("a_counter_value_cannot_inject_a_help_line", (1, 1, 3)),
    ("the_rendered_body_is_a_pure_function_of_the_counts", (True, True)),
    ("rendering_does_not_disturb_the_counters_it_read", (4, 0, 1)),
    ("the_order_is_fixed_by_the_crate_not_by_arrival_order", ('service_connections_accepted_total', 'service_connections_rejected_total', 'service_connections_closed_total')),
    ("every_series_is_declared_a_counter", ('counter', 'counter', 'counter')),
    ("no_rendered_line_is_blank", (9, False)),
    ("a_rejected_connection_is_never_also_counted_as_accepted", (0, 9, 0)),
    ("a_closed_connection_does_not_decrement_the_accepted_count", (5, 5, 0)),
    ("two_independent_providers_do_not_share_counter_state", (2, 0, 0)),
    ("the_help_text_is_the_crates_own", ('Total accepted service connections.', 'Total service connections rejected by admission.', 'Total completed or failed service connections.')),
)


def refusal(fn, *args, **kwargs) -> str:
    """The exception type a refusing call raises, or 'accepted' if it returns."""
    try:
        fn(*args, **kwargs)
    except Exception as exc:  # noqa: BLE001 — the type is the observation
        return type(exc).__name__
    return "accepted"


def metrics(accepted: int, rejected: int, closed: int) -> LifecycleMetrics:
    m = LifecycleMetrics()
    for _ in range(accepted):
        m.connection_accepted()
    for _ in range(rejected):
        m.connection_rejected()
    for _ in range(closed):
        m.connection_closed()
    return m


def verify_connection_lifecycle_metrics_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a provider with no samples renders an empty body rather than failing
    exp1 = CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[0][1]
    obs1 = (render(()), render(()) == '', refusal(render, ()))
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[0][0], "expected": exp1,
                   "observed": obs1, "passed": obs1 == exp1})

    # 2. a counter value cannot inject a HELP line, because it is an integer
    exp2 = CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[1][1]
    injected = render((Sample("a_total", "counter", "A.", 7),))
    obs2 = (injected.count('# HELP'), injected.count('# TYPE'), len(injected.splitlines()))
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[1][0], "expected": exp2,
                   "observed": obs2, "passed": obs2 == exp2})

    # 3. the rendered body is a pure function of the counts
    exp3 = CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[2][1]
    steady = metrics(3, 1, 2)
    obs3 = (steady.render_metrics() == steady.render_metrics(), metrics(3, 1, 2).render_metrics() == steady.render_metrics())
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[2][0], "expected": exp3,
                   "observed": obs3, "passed": obs3 == exp3})

    # 4. rendering does not disturb the counters it read
    exp4 = CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[3][1]
    watched = metrics(4, 0, 1)
    watched.render_metrics()
    watched.render_metrics()
    obs4 = (watched.accepted(), watched.rejected(), watched.closed())
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[3][0], "expected": exp4,
                   "observed": obs4, "passed": obs4 == exp4})

    # 5. the order is fixed by the crate, not by the order events arrived
    exp5 = CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[4][1]
    late = LifecycleMetrics()
    late.connection_closed()
    late.connection_rejected()
    late.connection_accepted()
    obs5 = tuple((line.split()[0] for line in late.render_metrics().splitlines() if not line.startswith('#')))
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[4][0], "expected": exp5,
                   "observed": obs5, "passed": obs5 == exp5})

    # 6. every series is declared a counter, never an untyped or gauge series
    exp6 = CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[5][1]
    obs6 = tuple((line.split()[3] for line in metrics(1, 1, 1).render_metrics().splitlines() if line.startswith('# TYPE')))
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[5][0], "expected": exp6,
                   "observed": obs6, "passed": obs6 == exp6})

    # 7. no rendered line is blank, so a scraper never sees a stray separator
    exp7 = CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[6][1]
    lines = metrics(2, 2, 2).render_metrics().splitlines()
    obs7 = (len(lines), any((line.strip() == '' for line in lines)))
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[6][0], "expected": exp7,
                   "observed": obs7, "passed": obs7 == exp7})

    # 8. a rejected connection is never also counted as accepted
    exp8 = CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[7][1]
    pressure = metrics(0, 9, 0)
    obs8 = (pressure.accepted(), pressure.rejected(), pressure.closed())
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[7][0], "expected": exp8,
                   "observed": obs8, "passed": obs8 == exp8})

    # 9. a closed connection does not decrement the accepted count
    exp9 = CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[8][1]
    churn = metrics(5, 0, 5)
    obs9 = (churn.accepted(), churn.closed(), churn.accepted() - churn.closed())
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[8][0], "expected": exp9,
                   "observed": obs9, "passed": obs9 == exp9})

    # 10. two independent providers do not share counter state
    exp10 = CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[9][1]
    a, b = LifecycleMetrics(), LifecycleMetrics()
    a.connection_accepted()
    a.connection_accepted()
    obs10 = (a.accepted(), b.accepted(), b.rejected())
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[9][0], "expected": exp10,
                   "observed": obs10, "passed": obs10 == exp10})

    # 11. the help text is the crate's own, so a dashboard reads every service
    exp11 = CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[10][1]
    obs11 = tuple((line.split(' ', 3)[3] for line in metrics(0, 0, 0).render_metrics().splitlines() if line.startswith('# HELP')))
    checks.append({"name": CONNECTION_LIFECYCLE_METRICS_SECURITY_MATRIX[10][0], "expected": exp11,
                   "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "connection-lifecycle-metrics-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks)
        and len(checks) >= MINIMUM_CHECKS,
    }
