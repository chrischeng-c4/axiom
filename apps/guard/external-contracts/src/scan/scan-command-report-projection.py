"""Behavior contract for the public Guard scan command projection."""

from guard_contract import assert_scan_consistency, fixture, run_guard

DIMENSION = "behavior"


def verify() -> list[str]:
    with fixture({"nested/safe.js": "const answer = 42;\n"}) as root:
        _, report = run_guard(
            ["scan", str(root), "--compact", "--no-persist"],
        )
        assertions = assert_scan_consistency(report)
        if report["target"] != str(root):
            raise AssertionError(
                f"scan target was not projected exactly: {report['target']!r}"
            )
        if report["policy_profile"] != "guard-baseline-static/1":
            raise AssertionError("scan did not project the default versioned policy")
        if report["integrations"].get("static_engine") != "compass":
            raise AssertionError("scan did not project the Compass integration")
        assertions.extend(
            [
                "scan projects the exact requested filesystem target",
                "scan projects guard-baseline-static/1 by default",
                "scan projects Compass as the static engine",
            ]
        )
        return assertions
