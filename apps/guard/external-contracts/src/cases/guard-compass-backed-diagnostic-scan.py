"""External contract for Compass-backed diagnostic normalization."""

from guard_contract import assert_finding, assert_scan_consistency, run_fixture_scan

DIMENSION = "security"


def verify() -> list[str]:
    report = run_fixture_scan(
        {"unsafe.js": "eval('alert(1)');\n"},
        expected_exit_codes={1},
    )
    assertions = assert_scan_consistency(report)
    finding = assert_finding(report, rule="JS004")
    location = finding.get("location", {})
    evidence = finding.get("evidence", {})
    if evidence.get("source") != "compass":
        raise AssertionError("diagnostic source is not Compass")
    if location.get("start_line") != 1 or not str(location.get("path", "")).endswith(
        "unsafe.js"
    ):
        raise AssertionError(f"diagnostic location was not preserved: {location!r}")
    if not str(finding.get("id", "")).startswith("compass:JS004:"):
        raise AssertionError("finding id did not preserve Compass rule identity")
    assertions.extend(
        [
            "one JS004 diagnostic becomes one Guard finding",
            "finding identity preserves Compass and JS004",
            "finding location points to line one of unsafe.js",
        ]
    )
    return assertions
