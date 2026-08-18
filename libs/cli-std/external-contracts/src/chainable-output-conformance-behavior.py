from __future__ import annotations

from cli_std.domain.chainable import (
    NextLine,
    assert_chainable,
    has_runnable_command,
    has_terminal_marker,
    trailing_next_line,
)

MINIMUM_CHECKS = 12

CHAINABLE_OUTPUT_CONFORMANCE_BEHAVIOR_MATRIX = [
    ("bare_next_string_command", True),
    ("nested_next_command", True),
    ("invoke_command", True),
    ("terminal_completion_workflow_complete", True),
    ("terminal_status_done", True),
    ("terminal_next_done_not_runnable", True),
    ("text_trailing_next_done", "done"),
    ("text_trailing_next_command", "command"),
    ("text_trailing_next_done_with_blank_lines", "done"),
    ("text_bare_next_prefix_not_command", None),
    ("text_marker_must_be_on_last_line", None),
    ("assert_chainable_passes_valid_terminal_json", None),
]


def verify_chainable_output_conformance_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    c0 = has_runnable_command({"next": "run-task"})
    checks.append({"name": "bare_next_string_command", "passed": c0 == True})

    c1 = has_runnable_command({"next": {"command": "run-nested"}})
    checks.append({"name": "nested_next_command", "passed": c1 == True})

    c2 = has_runnable_command({"invoke": {"command": "run-invoked"}})
    checks.append({"name": "invoke_command", "passed": c2 == True})

    c3 = has_terminal_marker({"completion": {"workflow_complete": True}})
    checks.append({"name": "terminal_completion_workflow_complete", "passed": c3 == True})

    c4 = has_terminal_marker({"status": "done"})
    checks.append({"name": "terminal_status_done", "passed": c4 == True})

    c5 = (
        has_terminal_marker({"next": "done"})
        and not has_runnable_command({"next": "done"})
    )
    checks.append({"name": "terminal_next_done_not_runnable", "passed": c5 == True})

    res6 = trailing_next_line("hello\nnext: done")
    c6 = res6.value if isinstance(res6, NextLine) else None
    checks.append({"name": "text_trailing_next_done", "passed": c6 == "done"})

    res7 = trailing_next_line("hello\nnext: do-something")
    c7 = res7.value if isinstance(res7, NextLine) else None
    checks.append({"name": "text_trailing_next_command", "passed": c7 == "command"})

    res8 = trailing_next_line("hello\nnext: done\n  \n")
    c8 = res8.value if isinstance(res8, NextLine) else None
    checks.append({"name": "text_trailing_next_done_with_blank_lines", "passed": c8 == "done"})

    res9 = trailing_next_line("hello\nnext:   ")
    c9 = res9.value if isinstance(res9, NextLine) else None
    checks.append({"name": "text_bare_next_prefix_not_command", "passed": c9 is None})

    res10 = trailing_next_line("next: done\nother line")
    c10 = res10.value if isinstance(res10, NextLine) else None
    checks.append({"name": "text_marker_must_be_on_last_line", "passed": c10 is None})

    fake_parse = lambda s: {"status": "done"} if s == '{"status":"done"}' else None
    res11 = assert_chainable('{"status":"done"}', fake_parse)
    c11 = res11.reason if hasattr(res11, "reason") else None
    checks.append({"name": "assert_chainable_passes_valid_terminal_json", "passed": c11 is None})

    return {
        "case_id": "chainable-output-conformance-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
