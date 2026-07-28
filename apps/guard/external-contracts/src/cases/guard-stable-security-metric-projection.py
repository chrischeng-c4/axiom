"""Stability contract for Guard's lifecycle-facing security metric."""

from guard_contract import assert_scan_consistency, run_fixture_scan

DIMENSION = "stability"


def _projection(report: dict[str, object]) -> tuple[object, ...]:
    summary = report["summary"]
    status = report["status"]
    completion = report["completion"]
    findings = report["findings"]
    assert isinstance(summary, dict)
    assert isinstance(status, dict)
    assert isinstance(completion, dict)
    assert isinstance(findings, list)
    return (
        report["schema_version"],
        status.get("state"),
        report["exit_code"],
        summary.get("security_findings"),
        completion.get("clean"),
        tuple(
            (item.get("rule"), item.get("severity"))
            for item in findings
            if isinstance(item, dict)
        ),
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
        raise AssertionError("equivalent scans changed the lifecycle security metric")
    assertions.append("equivalent scans preserve the lifecycle security projection")
    return assertions
