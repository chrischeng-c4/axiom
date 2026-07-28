"""External contract for the baseline static security policy."""

from guard_contract import assert_finding, assert_scan_consistency, run_fixture_scan

DIMENSION = "security"


def verify() -> list[str]:
    report = run_fixture_scan(
        {"unsafe.js": "eval('alert(1)');\n"},
        profile="baseline-static",
        expected_exit_codes={1},
    )
    assertions = assert_scan_consistency(report)
    finding = assert_finding(report, rule="JS004")
    if report["policy_profile"] != "guard-baseline-static/1":
        raise AssertionError("baseline-static did not select its versioned policy")
    if finding.get("severity") not in {"high", "medium", "low"}:
        raise AssertionError(f"JS004 was not actionable: {finding!r}")
    if "dynamic code execution" not in str(finding.get("remediation", "")):
        raise AssertionError("JS004 remediation is missing its stable action")
    if finding.get("evidence", {}).get("source") != "compass":
        raise AssertionError("JS004 did not preserve Compass source identity")
    assertions.extend(
        [
            "baseline-static selects guard-baseline-static/1",
            "JS004 is actionable and names dynamic-code remediation",
            "JS004 evidence source is Compass",
        ]
    )
    return assertions
