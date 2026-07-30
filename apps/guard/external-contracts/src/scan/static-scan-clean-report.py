"""External contract for the clean Guard report envelope."""

from guard_contract import (
    assert_scan_consistency,
    expected_static_engine,
    run_fixture_scan,
)

DIMENSION = "security"


def verify() -> list[str]:
    report = run_fixture_scan({"safe.js": "const answer = 42;\n"})
    assertions = assert_scan_consistency(report)
    if report["summary"]["security_findings"] != 0 or report["findings"]:
        raise AssertionError("safe fixture did not produce a zero-finding report")
    engine = expected_static_engine()
    if report["integrations"].get("static_engine") != engine:
        raise AssertionError(f"static engine identity is not {engine}")
    if report["completion"].get("clean") is not True:
        raise AssertionError("clean report did not mark completion clean")
    assertions.extend(
        [
            "safe static scan exits zero with no findings",
            f"static integration identity is honestly {engine}",
            "clean report marks completion clean",
        ]
    )
    return assertions
