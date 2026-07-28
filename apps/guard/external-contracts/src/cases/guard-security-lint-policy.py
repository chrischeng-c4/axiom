"""External contract for security-impacting lint policy selection."""

from guard_contract import assert_finding, assert_scan_consistency, run_fixture_scan

DIMENSION = "security"


def verify() -> list[str]:
    baseline = run_fixture_scan({"Dockerfile": "FROM ubuntu\n"})
    security_lint = run_fixture_scan(
        {"Dockerfile": "FROM ubuntu\n"},
        profile="security-lint",
        expected_exit_codes={1},
    )
    assertions = assert_scan_consistency(baseline)
    assertions.extend(assert_scan_consistency(security_lint))
    if baseline["summary"]["security_findings"] != 0:
        raise AssertionError("baseline unexpectedly included security-impacting lint")
    finding = assert_finding(security_lint, rule="DK002")
    if (
        security_lint["summary"]["security_findings"] != 1
        or len(security_lint["findings"]) != 1
    ):
        raise AssertionError("security-lint emitted findings beyond the one DK002 result")
    if security_lint["policy_profile"] != "guard-security-lint/1":
        raise AssertionError("security-lint did not select its versioned policy")
    if "Pin the image" not in str(finding.get("remediation", "")):
        raise AssertionError("DK002 remediation does not tell the user to pin the image")
    assertions.extend(
        [
            "baseline excludes DK002 security-impacting lint",
            "security-lint includes exactly one DK002 finding",
            "DK002 remediation tells the user to pin the image",
        ]
    )
    return assertions
