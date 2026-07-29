"""Behavior contract for exact Guard adapter routing and evidence folding."""

from guard_contract import assert_dynamic_evidence, run_dynamic_adapters

DIMENSION = "behavior"


def verify() -> list[str]:
    tools = ("vat", "rig", "meter")
    report, traces, expectations = run_dynamic_adapters(tools)
    assertions = assert_dynamic_evidence(
        report,
        traces,
        expectations=expectations,
    )
    expected_prefixes = {
        "vat": ["run", "--json", "guard-security-smoke"],
        "rig": ["run", "--scenario"],
        "meter": ["run", "--target"],
    }
    for tool, prefix in expected_prefixes.items():
        argv = traces[tool][0]
        if argv[: len(prefix)] != prefix:
            raise AssertionError(
                f"{tool} routing prefix diverged: {argv!r}"
            )
    assertions.append("all public adapter inputs map to their exact command grammar")
    return assertions
