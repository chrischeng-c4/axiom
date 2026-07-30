"""Behavior contract for consuming Guard's lifecycle security decision."""

from guard_contract import assert_scan_consistency, run_fixture_scan

DIMENSION = "behavior"


def _consumer_decision(report: dict[str, object]) -> str:
    summary = report["summary"]
    status = report["status"]
    completion = report["completion"]
    assert isinstance(summary, dict)
    assert isinstance(status, dict)
    assert isinstance(completion, dict)
    state = status.get("state")
    if (
        report["exit_code"] == 0
        and summary.get("security_findings") == 0
        and completion.get("clean") is True
        and state == "clean"
    ):
        return "clean"
    if (
        report["exit_code"] == 1
        and int(summary.get("security_findings", 0)) > 0
        and completion.get("clean") is False
        and state == "findings"
    ):
        return "findings"
    raise AssertionError(f"Guard report cannot be consumed unambiguously: {report!r}")


def verify() -> list[str]:
    clean = run_fixture_scan({"safe.js": "const answer = 42;\n"})
    findings = run_fixture_scan(
        {"unsafe.js": "eval('alert(1)');\n"},
        expected_exit_codes={1},
    )
    assertions = assert_scan_consistency(clean)
    assertions.extend(assert_scan_consistency(findings))
    if _consumer_decision(clean) != "clean":
        raise AssertionError("clean report did not project a clean consumer decision")
    if _consumer_decision(findings) != "findings":
        raise AssertionError("finding report did not project a findings decision")
    if not clean["agent_prompt"] or clean["agent_prompt"] == findings["agent_prompt"]:
        raise AssertionError("agent_prompt did not explain the changed decision")
    assertions.extend(
        [
            "public fields derive an unambiguous clean consumer decision",
            "public fields derive an unambiguous findings consumer decision",
            "agent_prompt distinguishes the two lifecycle decisions",
        ]
    )
    return assertions
