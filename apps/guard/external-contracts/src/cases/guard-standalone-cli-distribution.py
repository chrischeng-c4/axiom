"""External contract for Guard's independently built CLI distribution."""

import subprocess

from guard_contract import (
    assert_report_shape,
    assert_scan_consistency,
    build_standalone_guard,
    fixture,
    run_guard,
)

DIMENSION = "behavior"


def verify() -> list[str]:
    binary = build_standalone_guard()
    help_result = subprocess.run(
        [str(binary), "--help"],
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
            "an override-free isolated cargo build succeeds for package guard and binary guard",
            "public help exposes scan, report, spec, and llm from the built binary",
            "scan persists a clean guard.report/1 report and report re-projects it exactly",
            "offline spec identifies guard.report/1 and llm teaches guard scan",
        ]
    )
    return assertions
