"""Stability contract for Compass-to-Guard diagnostic normalization."""

from guard_contract import assert_finding, assert_scan_consistency, run_fixture_scan

DIMENSION = "stability"


def _projection(report: dict[str, object]) -> tuple[object, ...]:
    finding = assert_finding(report, rule="JS004")
    evidence = finding.get("evidence")
    location = finding.get("location")
    assert isinstance(evidence, dict)
    assert isinstance(location, dict)
    return (
        report["schema_version"],
        report["status"].get("state"),
        report["summary"].get("security_findings"),
        finding.get("rule"),
        finding.get("severity"),
        evidence.get("source"),
        location.get("start_line"),
    )


def verify() -> list[str]:
    first = run_fixture_scan(
        {"unsafe.js": "eval('alert(1)');\n"},
        expected_exit_codes={1},
    )
    second = run_fixture_scan(
        {"unsafe.js": "eval('alert(1)');\n"},
        expected_exit_codes={1},
    )
    assertions = assert_scan_consistency(first)
    assertions.extend(assert_scan_consistency(second))
    if _projection(first) != _projection(second):
        raise AssertionError("equivalent diagnostics changed normalized public fields")
    assertions.append("Compass diagnostic normalization is stable across fresh fixtures")
    return assertions
