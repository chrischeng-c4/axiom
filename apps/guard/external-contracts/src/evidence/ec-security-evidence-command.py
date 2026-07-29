"""External contract for composed vat, rig, and meter security evidence."""

from guard_contract import (
    AdapterOutcome,
    assert_dynamic_evidence,
    assert_finding,
    run_dynamic_adapters,
)

DIMENSION = "security"


def verify() -> list[str]:
    tools = ("vat", "rig", "meter")
    report, traces, expectations = run_dynamic_adapters(tools)
    assertions = assert_dynamic_evidence(
        report,
        traces,
        expectations=expectations,
    )
    if report["completion"]["missing"]:
        raise AssertionError(
            f"fully configured evidence still reported missing adapters: "
            f"{report['completion']['missing']!r}"
        )
    assertions.append("fully configured vat, rig, and meter leaves no missing adapter")
    failed, failed_traces, failed_expectations = run_dynamic_adapters(
        ("rig",),
        outcomes={"rig": AdapterOutcome(False, 1, 2)},
    )
    assertions.extend(
        assert_dynamic_evidence(
            failed,
            failed_traces,
            expectations=failed_expectations,
        )
    )
    evidence = failed["evidence"]
    summary = failed["summary"]
    assert isinstance(evidence, list)
    assert isinstance(summary, dict)
    if failed["exit_code"] != 1 or failed["status"].get("state") != "findings":
        raise AssertionError("non-clean Rig adapter did not fail the Guard report")
    if summary.get("evidence_failed") != 1 or evidence[0].get("finding_count") != 2:
        raise AssertionError("non-clean Rig counts were not folded into Guard")
    finding = assert_finding(failed, rule="RIG-EVIDENCE")
    if finding.get("evidence", {}).get("report", {}).get("clean") is not False:
        raise AssertionError("Guard finding did not preserve the non-clean Rig report")
    assertions.extend(
        [
            "non-clean Rig subprocess makes the public Guard report fail closed",
            "evidence_failed and finding_count preserve the external finding count",
            "RIG-EVIDENCE preserves the folded non-clean report",
        ]
    )
    exit_only, exit_traces, exit_expectations = run_dynamic_adapters(
        ("rig",),
        outcomes={"rig": AdapterOutcome(True, 7, 0)},
    )
    assertions.extend(
        assert_dynamic_evidence(
            exit_only,
            exit_traces,
            expectations=exit_expectations,
        )
    )
    exit_evidence = exit_only["evidence"][0]
    if (
        exit_evidence["exit_code"] != 7
        or exit_evidence["report"]["clean"] is not True
        or exit_evidence["finding_count"] != 0
    ):
        raise AssertionError("process-only failure channels were not preserved independently")
    report_only, report_traces, report_expectations = run_dynamic_adapters(
        ("rig",),
        outcomes={"rig": AdapterOutcome(False, 0, 2)},
    )
    assertions.extend(
        assert_dynamic_evidence(
            report_only,
            report_traces,
            expectations=report_expectations,
        )
    )
    report_evidence = report_only["evidence"][0]
    if (
        report_evidence["exit_code"] != 0
        or report_evidence["report"]["clean"] is not False
        or report_evidence["finding_count"] != 2
    ):
        raise AssertionError("report-only failure channels were not preserved independently")
    zero_count, zero_count_traces, zero_count_expectations = run_dynamic_adapters(
        ("rig",),
        outcomes={"rig": AdapterOutcome(False, 0, 0)},
    )
    assertions.extend(
        assert_dynamic_evidence(
            zero_count,
            zero_count_traces,
            expectations=zero_count_expectations,
        )
    )
    zero_count_evidence = zero_count["evidence"][0]
    if (
        zero_count["exit_code"] != 1
        or zero_count["status"].get("state") != "findings"
        or zero_count_evidence["exit_code"] != 0
        or zero_count_evidence["report"]["clean"] is not False
        or zero_count_evidence["finding_count"] != 0
        or zero_count_evidence["report"]["findings_preview"] != []
    ):
        raise AssertionError(
            "finding count did not vary independently of exit and report state"
        )
    assertions.extend(
        [
            "a nonzero adapter exit fails closed even when its report says clean",
            "a non-clean adapter report fails closed even when its process exits zero",
            "a non-clean zero-finding report fails closed while preserving zero findings",
            "process exit, report clean state, and finding count remain independent",
        ]
    )
    return assertions
