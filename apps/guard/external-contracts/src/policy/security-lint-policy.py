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
    strict = run_fixture_scan(
        {"Dockerfile": "FROM ubuntu\n"},
        profile="strict",
        expected_exit_codes={1},
    )
    assertions = assert_scan_consistency(baseline)
    assertions.extend(assert_scan_consistency(security_lint))
    assertions.extend(assert_scan_consistency(strict))
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
    strict_finding = assert_finding(strict, rule="DK002")
    if finding.get("severity") != "medium":
        raise AssertionError("security-lint DK002 must be medium severity")
    if strict_finding.get("severity") != "high":
        raise AssertionError("strict DK002 must be high severity")
    if strict["policy_profile"] != "guard-strict/1":
        raise AssertionError("strict did not select its versioned policy")
    if (
        strict["summary"]["security_findings"] != 1
        or len(strict["findings"]) != 1
    ):
        raise AssertionError("strict emitted findings beyond the one DK002 result")
    assertions.extend(
        [
            "baseline excludes DK002 security-impacting lint",
            "security-lint includes exactly one DK002 finding",
            "DK002 remediation tells the user to pin the image",
            "strict selects guard-strict/1 and promotes the sole DK002 to high",
        ]
    )
    return assertions
