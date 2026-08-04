from __future__ import annotations

from metrics_prometheus.application.exposition import render_labeled
from metrics_prometheus.domain.escaping import escape_label_value
from metrics_prometheus.domain.sample import Label, LabeledSample, MetricKind, SampleGroup

MINIMUM_CHECKS = 14

LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX = (
    ("escape_bare_quote", '\\"'),
    ("escape_bare_backslash", "\\\\"),
    ("escape_newline", "\\n"),
    ("escape_backslash_then_quote", '\\\\\\"'),
    ("escape_injection_payload", 'x\\"} evil_metric 1\\n'),
    ("render_injection_sample_line_count", 3),
    ("render_injection_unescaped_quote_count", 2),
    ("render_injection_sample_top_level_comma_count", 0),
    ("render_injection_no_evil_metric_line", 0),
    ("render_newline_sample_real_newline_count", 1),
    ("render_quote_sample_line_literal", 'http_requests{path="a\\"b"} 1\n'),
    ("escape_multiple_quotes", 'a\\"b\\"c\\"'),
    ("escape_already_escaped_looking_input", '\\\\\\"'),
    ("render_five_payloads_declared_separator_counts", (0, 0, 0, 0, 0)),
)


def verify_label_value_containment_security() -> dict[str, object]:
    checks = []

    p1 = '"'
    p2 = "\\"
    p3 = "\n"
    p4 = '\\"'
    p5 = 'x"} evil_metric 1\n'

    # 1. escape_bare_quote
    exp1 = LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[0][1]
    obs1 = escape_label_value(p1)
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[0][0],
        "expected": exp1,
        "observed": obs1,
        "passed": obs1 == exp1,
    })

    # 2. escape_bare_backslash
    exp2 = LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[1][1]
    obs2 = escape_label_value(p2)
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[1][0],
        "expected": exp2,
        "observed": obs2,
        "passed": obs2 == exp2,
    })

    # 3. escape_newline
    exp3 = LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[2][1]
    obs3 = escape_label_value(p3)
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[2][0],
        "expected": exp3,
        "observed": obs3,
        "passed": obs3 == exp3 and "\n" not in obs3,
    })

    # 4. escape_backslash_then_quote
    exp4 = LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[3][1]
    obs4 = escape_label_value(p4)
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[3][0],
        "expected": exp4,
        "observed": obs4,
        "passed": obs4 == exp4,
    })

    # 5. escape_injection_payload
    exp5 = LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[4][1]
    obs5 = escape_label_value(p5)
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[4][0],
        "expected": exp5,
        "observed": obs5,
        "passed": obs5 == exp5,
    })

    # 6. render_injection_sample_line_count
    exp6 = LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[5][1]
    g5 = SampleGroup("g5", MetricKind.COUNTER, "h", (LabeledSample((Label("k", p5),), 1),))
    rendered_g5 = render_labeled((g5,))
    lines_g5 = [ln for ln in rendered_g5.split("\n") if ln]
    obs6 = len(lines_g5)
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[5][0],
        "expected": exp6,
        "observed": obs6,
        "passed": obs6 == exp6,
    })

    # 7. render_injection_unescaped_quote_count
    exp7 = LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[6][1]
    sample_line_g5 = lines_g5[-1]
    unescaped = 0
    for i, ch in enumerate(sample_line_g5):
        if ch == '"' and (i == 0 or sample_line_g5[i - 1] != "\\"):
            unescaped += 1
    obs7 = unescaped
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[6][0],
        "expected": exp7,
        "observed": obs7,
        "passed": obs7 == exp7,
    })

    # 8. render_injection_sample_top_level_comma_count
    exp8 = LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[7][1]
    brace_content = sample_line_g5[sample_line_g5.find("{") + 1 : sample_line_g5.rfind("}")]
    obs8 = brace_content.count(",")
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[7][0],
        "expected": exp8,
        "observed": obs8,
        "passed": obs8 == exp8,
    })

    # 9. render_injection_no_evil_metric_line
    exp9 = LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[8][1]
    obs9 = sum(1 for ln in lines_g5 if ln.startswith("evil_metric"))
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[8][0],
        "expected": exp9,
        "observed": obs9,
        "passed": obs9 == exp9,
    })

    # 10. render_newline_sample_real_newline_count
    exp10 = LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[9][1]
    g3 = SampleGroup("g3", MetricKind.COUNTER, "h", (LabeledSample((Label("k", p3),), 1),))
    rendered_g3_raw = render_labeled((g3,)).splitlines(keepends=True)[-1]
    obs10 = rendered_g3_raw.count("\n")
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[9][0],
        "expected": exp10,
        "observed": obs10,
        "passed": obs10 == exp10,
    })

    # 11. render_quote_sample_line_literal
    exp11 = LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[10][1]
    g_quote = SampleGroup("http_requests", MetricKind.COUNTER, "h", (LabeledSample((Label("path", 'a"b'),), 1),))
    obs11 = render_labeled((g_quote,)).splitlines(keepends=True)[-1]
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[10][0],
        "expected": exp11,
        "observed": obs11,
        "passed": obs11 == exp11,
    })

    # 12. escape_multiple_quotes
    exp12 = LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[11][1]
    obs12 = escape_label_value('a"b"c"')
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[11][0],
        "expected": exp12,
        "observed": obs12,
        "passed": obs12 == exp12,
    })

    # 13. escape_already_escaped_looking_input
    exp13 = LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[12][1]
    obs13 = escape_label_value('\\"')
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[12][0],
        "expected": exp13,
        "observed": obs13,
        "passed": obs13 == exp13 and obs13 != '\\"',
    })

    # 14. render_five_payloads_declared_separator_counts
    exp14 = LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[13][1]
    sep_counts = []
    for payload in (p1, p2, p3, p4, p5):
        grp = SampleGroup("g", MetricKind.COUNTER, "h", (LabeledSample((Label("k", payload),), 1),))
        line = render_labeled((grp,)).splitlines()[-1]
        b_text = line[line.find("{") + 1 : line.rfind("}")]
        sep_counts.append(b_text.count(","))
    obs14 = tuple(sep_counts)
    checks.append({
        "name": LABEL_VALUE_CONTAINMENT_SECURITY_MATRIX[13][0],
        "expected": exp14,
        "observed": obs14,
        "passed": obs14 == exp14,
    })

    return {
        "case_id": "label-value-containment-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
