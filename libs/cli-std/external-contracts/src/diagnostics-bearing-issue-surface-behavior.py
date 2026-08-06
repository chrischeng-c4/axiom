from __future__ import annotations

from cli_std.domain.issue_body import (
    assemble_body,
    followup_comment_body,
    issue_payload,
    render_diagnostics,
    report_labels,
    resolve_repo,
)
from cli_std.domain.tool_identity import ToolInfo

MINIMUM_CHECKS = 12

DIAGNOSTICS_BEARING_ISSUE_SURFACE_BEHAVIOR_MATRIX = [
    (
        "render_diagnostics_without_node_exact_string",
        "## Diagnostics\n- mytool version: 1.0.0\n- target: x86_64-mac\n- git sha: abc1234\n- built at: 2026-01-01\n- os/arch: darwin/x86_64\n",
    ),
    (
        "render_diagnostics_with_node_exact_string",
        "## Diagnostics\n- mytool version: 1.0.0\n- target: x86_64-mac\n- git sha: abc1234\n- built at: 2026-01-01\n- os/arch: darwin/x86_64\n- node: node-1\n",
    ),
    (
        "assemble_body_with_message",
        "Bug happened\n\n---\n## Diagnostics\n- mytool version: 1.0.0\n- target: x86_64-mac\n- git sha: abc1234\n- built at: 2026-01-01\n- os/arch: darwin/x86_64\n",
    ),
    ("resolve_repo_default", "owner/mytool"),
    ("resolve_repo_override", "other/repo"),
    ("issue_payload_omits_labels_when_empty", {"title": "T", "body": "B"}),
    (
        "issue_payload_includes_labels_when_present",
        {"title": "T", "body": "B", "labels": ["l1"]},
    ),
    (
        "report_labels_adds_canonical_labels",
        ("app:mytool", "type:report"),
    ),
    (
        "report_labels_does_not_duplicate_existing_tool_label",
        ("app:mytool", "custom", "type:report"),
    ),
    (
        "report_labels_includes_type_report",
        ("type:report", "app:mytool"),
    ),
    (
        "followup_comment_body_default_message",
        "User-side verification failed after closure; reopening for follow-up.\n\n---\n## Diagnostics\n- mytool version: 1.0.0\n- target: x86_64-mac\n- git sha: abc1234\n- built at: 2026-01-01\n- os/arch: darwin/x86_64\n",
    ),
    (
        "followup_comment_body_omits_node",
        "Custom message\n\n---\n## Diagnostics\n- mytool version: 1.0.0\n- target: x86_64-mac\n- git sha: abc1234\n- built at: 2026-01-01\n- os/arch: darwin/x86_64\n",
    ),
]


def verify_diagnostics_bearing_issue_surface_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    tool = ToolInfo(
        project="mytool",
        repo="owner/mytool",
        target="x86_64-mac",
        version="1.0.0",
        git_sha="abc1234",
        built_at="2026-01-01",
    )

    c0 = render_diagnostics(tool, "darwin", "x86_64", None)
    expected_diag = (
        "## Diagnostics\n"
        "- mytool version: 1.0.0\n"
        "- target: x86_64-mac\n"
        "- git sha: abc1234\n"
        "- built at: 2026-01-01\n"
        "- os/arch: darwin/x86_64\n"
    )
    checks.append(
        {
            "name": "render_diagnostics_without_node_exact_string",
            "passed": c0 == expected_diag,
        }
    )

    c1 = render_diagnostics(tool, "darwin", "x86_64", "node-1")
    expected_diag_node = (
        "## Diagnostics\n"
        "- mytool version: 1.0.0\n"
        "- target: x86_64-mac\n"
        "- git sha: abc1234\n"
        "- built at: 2026-01-01\n"
        "- os/arch: darwin/x86_64\n"
        "- node: node-1\n"
    )
    checks.append(
        {
            "name": "render_diagnostics_with_node_exact_string",
            "passed": c1 == expected_diag_node,
        }
    )

    c2 = assemble_body("Bug happened", expected_diag)
    checks.append(
        {
            "name": "assemble_body_with_message",
            "passed": c2 == f"Bug happened\n\n---\n{expected_diag}",
        }
    )

    c3 = resolve_repo(tool, None)
    checks.append({"name": "resolve_repo_default", "passed": c3 == "owner/mytool"})

    c4 = resolve_repo(tool, "other/repo")
    checks.append({"name": "resolve_repo_override", "passed": c4 == "other/repo"})

    c5 = issue_payload("T", "B", [])
    checks.append(
        {
            "name": "issue_payload_omits_labels_when_empty",
            "passed": c5 == {"title": "T", "body": "B"},
        }
    )

    c6 = issue_payload("T", "B", ["l1"])
    checks.append(
        {
            "name": "issue_payload_includes_labels_when_present",
            "passed": c6 == {"title": "T", "body": "B", "labels": ["l1"]},
        }
    )

    c7 = report_labels(tool, [])
    checks.append(
        {
            "name": "report_labels_adds_canonical_labels",
            "passed": c7 == ("app:mytool", "type:report"),
        }
    )

    c8 = report_labels(tool, ["app:mytool", "custom"])
    checks.append(
        {
            "name": "report_labels_does_not_duplicate_existing_tool_label",
            "passed": c8 == ("app:mytool", "custom", "type:report"),
        }
    )

    c9 = report_labels(tool, ["type:report"])
    checks.append(
        {
            "name": "report_labels_includes_type_report",
            "passed": c9 == ("type:report", "app:mytool"),
        }
    )

    c10 = followup_comment_body(tool, None, "darwin", "x86_64")
    expected_followup = f"User-side verification failed after closure; reopening for follow-up.\n\n---\n{expected_diag}"
    checks.append(
        {
            "name": "followup_comment_body_default_message",
            "passed": c10 == expected_followup,
        }
    )

    c11 = followup_comment_body(tool, "Custom message", "darwin", "x86_64")
    expected_custom_followup = f"Custom message\n\n---\n{expected_diag}"
    checks.append(
        {
            "name": "followup_comment_body_omits_node",
            "passed": c11 == expected_custom_followup,
        }
    )

    return {
        "case_id": "diagnostics-bearing-issue-surface-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
