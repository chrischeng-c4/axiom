"""Behavior contract for every public Guard adapter route and evidence fold."""

from guard_contract import (
    assert_dynamic_evidence,
    run_dynamic_adapters,
    verify_adapter_route,
)

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
    for route in (
        "vat-command",
        "rig-dir",
        "rig-command",
        "meter-command",
        "arena-spec",
        "arena-command",
    ):
        assertions.extend(verify_adapter_route(route))
    assertions.append(
        "all nine public adapter flags map to their exact command grammar"
    )
    return assertions
