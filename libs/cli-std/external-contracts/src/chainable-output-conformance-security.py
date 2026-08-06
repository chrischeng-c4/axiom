from __future__ import annotations

from cli_std.domain.chainable import (
    NextLine,
    assert_chainable,
    has_runnable_command,
    has_terminal_marker,
    trailing_next_line,
)
from cli_std.domain.errors import ChainableViolation

MINIMUM_CHECKS = 11

CHAINABLE_OUTPUT_CONFORMANCE_SECURITY_MATRIX = [
    ("truthy_int_completion_refused", False),
    ("false_boolean_completion_refused", False),
    ("blank_next_command_refused", False),
    ("blank_invoke_command_refused", False),
    ("empty_output_returns_violation", "output is empty - no JSON payload and no `next:` line"),
    ("whitespace_output_returns_violation", "output is empty - no JSON payload and no `next:` line"),
    ("inert_json_returns_violation", "JSON payload has neither runnable command nor terminal marker"),
    ("inert_json_with_text_marker_returns_violation", "JSON payload has neither runnable command nor terminal marker"),
    ("non_json_text_without_next_returns_violation", "output is not valid JSON and has no trailing `next:` marker"),
    ("first_line_marker_ignored", (None, None)),
    ("blank_nested_next_command_refused", False),
]


def verify_chainable_output_conformance_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    fake_parse = lambda s: (
        {"foo": "bar"}
        if s == '{"foo":"bar"}'
        else (
            {"foo": "next: done"}
            if s == '{"foo": "next: done"}'
            else None
        )
    )

    c0 = has_terminal_marker({"completion": {"workflow_complete": 1}})
    checks.append({"name": "truthy_int_completion_refused", "passed": c0 == False})

    c1 = has_terminal_marker({"completion": {"workflow_complete": False}})
    checks.append({"name": "false_boolean_completion_refused", "passed": c1 == False})

    c2 = has_runnable_command({"next": "   "})
    checks.append({"name": "blank_next_command_refused", "passed": c2 == False})

    c3 = has_runnable_command({"invoke": {"command": "   "}})
    checks.append({"name": "blank_invoke_command_refused", "passed": c3 == False})

    res4 = assert_chainable("", fake_parse)
    c4 = res4.reason if isinstance(res4, ChainableViolation) else None
    checks.append({"name": "empty_output_returns_violation", "passed": c4 == "output is empty - no JSON payload and no `next:` line"})

    res5 = assert_chainable("   \n ", fake_parse)
    c5 = res5.reason if isinstance(res5, ChainableViolation) else None
    checks.append({"name": "whitespace_output_returns_violation", "passed": c5 == "output is empty - no JSON payload and no `next:` line"})

    res6 = assert_chainable('{"foo":"bar"}', fake_parse)
    c6 = res6.reason if isinstance(res6, ChainableViolation) else None
    checks.append({"name": "inert_json_returns_violation", "passed": c6 == "JSON payload has neither runnable command nor terminal marker"})

    res7 = assert_chainable('{"foo": "next: done"}', fake_parse)
    c7 = res7.reason if isinstance(res7, ChainableViolation) else None
    checks.append({"name": "inert_json_with_text_marker_returns_violation", "passed": c7 == "JSON payload has neither runnable command nor terminal marker"})

    res8 = assert_chainable("plain log output", fake_parse)
    c8 = res8.reason if isinstance(res8, ChainableViolation) else None
    checks.append({"name": "non_json_text_without_next_returns_violation", "passed": c8 == "output is not valid JSON and has no trailing `next:` marker"})

    res9_a = trailing_next_line("next: done\nsecond line")
    c9_a = res9_a.value if isinstance(res9_a, NextLine) else None
    res9_b = trailing_next_line("next: ")
    c9_b = res9_b.value if isinstance(res9_b, NextLine) else None
    checks.append({"name": "first_line_marker_ignored", "passed": (c9_a, c9_b) == (None, None)})

    c10 = has_runnable_command({"next": {"command": "   "}})
    checks.append({"name": "blank_nested_next_command_refused", "passed": c10 == False})

    return {
        "case_id": "chainable-output-conformance-security",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
