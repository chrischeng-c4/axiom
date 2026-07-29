"""External contract for Guard's independently built CLI distribution."""

import os
import subprocess

from guard_contract import (
    assert_report_shape,
    assert_scan_consistency,
    build_standalone_guard,
    fixture,
    guard_process_command,
    run_guard,
)

DIMENSION = "behavior"


def verify() -> list[str]:
    td_stage = os.environ.get("AW_EC_STAGE") == "td"
    binary = None if td_stage else build_standalone_guard()
    help_result = subprocess.run(
        [*guard_process_command(binary), "--help"],
        capture_output=True,
        text=True,
        check=False,
    )
    if help_result.returncode != 0:
        raise AssertionError(f"guard --help failed: {help_result.stderr!r}")
    for command in ("scan", "report", "spec", "llm"):
        if command not in help_result.stdout:
            raise AssertionError(f"guard --help omitted {command!r}")

    with fixture({"safe.js": "const answer = 42;\n"}) as root:
        _, scan_report = run_guard(
            ["scan", str(root), "--compact"],
            binary=binary,
            cwd=root,
        )
        assertions = assert_scan_consistency(scan_report)
        _, persisted_report = run_guard(
            ["report", "--compact"],
            binary=binary,
            cwd=root,
        )
        assertions.extend(assert_report_shape(persisted_report, verb="scan"))
        if persisted_report != scan_report:
            raise AssertionError("guard report did not re-project the persisted scan")

    with fixture({"Dockerfile": "FROM ubuntu:latest\n"}) as root:
        _, security_lint = run_guard(
            [
                "scan",
                str(root),
                "--profile",
                "security-lint",
                "--compact",
                "--no-persist",
            ],
            binary=binary,
            expected_exit_codes={1},
        )
        _, strict = run_guard(
            [
                "scan",
                str(root),
                "--profile",
                "strict",
                "--compact",
                "--no-persist",
            ],
            binary=binary,
            expected_exit_codes={1},
        )
    if security_lint["policy_profile"] != "guard-security-lint/1":
        raise AssertionError("security-lint profile identity drifted")
    if strict["policy_profile"] != "guard-strict/1":
        raise AssertionError("strict profile identity drifted")
    lint_finding = next(
        item for item in security_lint["findings"] if item.get("rule") == "DK002"
    )
    strict_finding = next(
        item for item in strict["findings"] if item.get("rule") == "DK002"
    )
    if lint_finding.get("severity") != "medium":
        raise AssertionError("security-lint DK002 must remain medium severity")
    if strict_finding.get("severity") != "high":
        raise AssertionError("strict must promote DK002 to high severity")

    _, spec_report = run_guard(["spec", "--compact"], binary=binary)
    assertions.extend(assert_report_shape(spec_report, verb="spec"))
    if "guard.report/1" not in spec_report["agent_prompt"]:
        raise AssertionError("guard spec did not identify the public report protocol")
    _, llm_report = run_guard(["llm", "--compact"], binary=binary)
    assertions.extend(assert_report_shape(llm_report, verb="llm"))
    if "guard scan" not in llm_report["agent_prompt"]:
        raise AssertionError("guard llm did not teach the public scan command")
    assertions.extend(
        [
            (
                "the Python TD runs as a standalone reference CLI"
                if td_stage
                else "an override-free isolated cargo build succeeds for package guard and binary guard"
            ),
            "public help exposes scan, report, spec, and llm from the built binary",
            "scan persists a clean guard.report/1 report and report re-projects it exactly",
            "strict is a real CLI policy and promotes DK002 from medium to high",
            "offline spec identifies guard.report/1 and llm teaches guard scan",
        ]
    )
    return assertions
