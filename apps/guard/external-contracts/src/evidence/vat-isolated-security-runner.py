"""External contract for the VAT isolated-runner adapter."""

from guard_contract import assert_dynamic_evidence, run_dynamic_adapters

DIMENSION = "security"


def verify() -> list[str]:
    report, traces, expectations = run_dynamic_adapters(("vat",))
    assertions = assert_dynamic_evidence(
        report,
        traces,
        expectations=expectations,
    )
    argv = traces["vat"][0]
    for token in ("run", "--json", "guard-security-smoke"):
        if token not in argv:
            raise AssertionError(f"VAT argv omitted {token!r}: {argv!r}")
    assertions.append("vat-runner maps to the declared public VAT runner id")
    return assertions
