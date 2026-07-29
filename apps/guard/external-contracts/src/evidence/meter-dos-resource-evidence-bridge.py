"""External contract for the Meter resource-evidence adapter."""

from guard_contract import assert_dynamic_evidence, run_dynamic_adapters

DIMENSION = "security"


def verify() -> list[str]:
    report, traces, expectations = run_dynamic_adapters(("meter",))
    assertions = assert_dynamic_evidence(
        report,
        traces,
        expectations=expectations,
    )
    argv = traces["meter"][0]
    for token in ("run", "--target", "--skip-bench", "--skip-profile", "--compact"):
        if token not in argv:
            raise AssertionError(f"Meter argv omitted {token!r}: {argv!r}")
    assertions.append("meter-target maps to the public Meter resource command")
    return assertions
