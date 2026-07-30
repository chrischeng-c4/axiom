"""External contract for Guard's lifecycle-consumable security metric."""

from guard_contract import assert_finding, assert_scan_consistency, run_fixture_scan

DIMENSION = "security"


def verify() -> list[str]:
    clean = run_fixture_scan({"safe.js": "const answer = 42;\n"})
    finding = run_fixture_scan(
        {"unsafe.js": "eval('alert(1)');\n"},
        expected_exit_codes={1},
    )
    assertions = assert_scan_consistency(clean)
    assertions.extend(assert_scan_consistency(finding))
    assert_finding(finding, rule="JS004")
    if clean["summary"]["security_findings"] != 0:
        raise AssertionError("clean fixture produced security findings")
    if finding["summary"]["security_findings"] != 1:
        raise AssertionError("vulnerable fixture did not produce one security finding")
    if clean["agent_prompt"] == finding["agent_prompt"]:
        raise AssertionError("agent_prompt did not reflect changed security state")
    assertions.extend(
        [
            "clean input projects a zero-finding clean metric",
            "vulnerable input projects a one-finding failed metric",
            "agent_prompt changes with the security state",
        ]
    )
    return assertions
