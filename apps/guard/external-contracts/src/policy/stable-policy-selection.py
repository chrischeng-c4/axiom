"""Stability contract for Guard policy selection and actionable findings."""

from guard_contract import assert_finding, assert_scan_consistency, run_fixture_scan

DIMENSION = "stability"


def _projection(report: dict[str, object]) -> tuple[object, ...]:
    finding = assert_finding(report, rule="DK002")
    evidence = finding.get("evidence")
    assert isinstance(evidence, dict)
    return (
        report["policy_profile"],
        finding.get("rule"),
        finding.get("severity"),
        finding.get("remediation"),
        evidence.get("source"),
    )


def verify() -> list[str]:
    first = run_fixture_scan(
        {"Dockerfile": "FROM ubuntu\n"},
        profile="security-lint",
        expected_exit_codes={1},
    )
    second = run_fixture_scan(
        {"Dockerfile": "FROM ubuntu\n"},
        profile="security-lint",
        expected_exit_codes={1},
    )
    assertions = assert_scan_consistency(first)
    assertions.extend(assert_scan_consistency(second))
    if _projection(first) != _projection(second):
        raise AssertionError("equivalent scans changed policy or actionable finding fields")
    assertions.append("security-lint policy and DK002 action remain stable across runs")
    return assertions
