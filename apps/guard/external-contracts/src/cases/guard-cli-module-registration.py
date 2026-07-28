"""External contract for Guard public command discovery."""

import subprocess

from guard_contract import assert_report_shape, guard_binary, run_guard

DIMENSION = "behavior"


def verify() -> list[str]:
    help_result = subprocess.run(
        [str(guard_binary()), "--help"],
        capture_output=True,
        text=True,
        check=False,
    )
    if help_result.returncode != 0:
        raise AssertionError(f"guard --help failed: {help_result.stderr!r}")
    for command in ("scan", "report", "spec", "llm"):
        if command not in help_result.stdout:
            raise AssertionError(f"guard --help omitted {command!r}")
    _, report = run_guard(["spec", "--compact"])
    assertions = assert_report_shape(report, verb="spec")
    if "guard.report/1" not in report["agent_prompt"]:
        raise AssertionError("guard spec did not identify the public report protocol")
    assertions.extend(
        [
            "public help exposes scan, report, spec, and llm",
            "offline spec identifies guard.report/1",
        ]
    )
    return assertions
