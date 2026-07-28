"""External contract for the clean Guard report envelope."""

from guard_contract import assert_scan_consistency, run_fixture_scan

DIMENSION = "security"


def verify() -> list[str]:
    report = run_fixture_scan({"safe.js": "const answer = 42;\n"})
    assertions = assert_scan_consistency(report)
    if report["summary"]["security_findings"] != 0 or report["findings"]:
        raise AssertionError("safe fixture did not produce a zero-finding report")
    if report["integrations"].get("static_engine") != "compass":
        raise AssertionError("static engine identity is not Compass")
    if report["completion"].get("clean") is not True:
        raise AssertionError("clean report did not mark completion clean")
    assertions.extend(
        [
            "safe static scan exits zero with no findings",
            "static integration identity is Compass",
            "clean report marks completion clean",
        ]
    )
    return assertions
