from __future__ import annotations

from cli_std.domain.issue_body import (
    assemble_body,
    comment_payload,
    percent_encode_query,
    prefilled_url,
)
from cli_std.domain.tool_identity import ToolInfo

MINIMUM_CHECKS = 12

DIAGNOSTICS_BEARING_ISSUE_SURFACE_SECURITY_MATRIX = [
    ("percent_encode_query_reserved_characters_uppercase_hex", "%26%20%23%3D%0A"),
    ("percent_encode_query_tilde_is_unreserved", "~"),
    ("percent_encode_query_multibyte_utf8", "%E2%82%AC"),
    ("assemble_body_none_message_yields_diagnostics_alone", "diag"),
    (
        "assemble_body_blank_whitespace_message_yields_diagnostics_alone",
        "diag",
    ),
    (
        "prefilled_url_percent_encodes_title_and_body",
        "https://github.com/owner/repo/issues/new?title=T%20%26%20T&body=B%20%23%20B",
    ),
    (
        "prefilled_url_joins_labels_with_comma",
        "https://github.com/owner/repo/issues/new?title=T&body=B&labels=a%2Cb",
    ),
    (
        "percent_encode_query_unreserved_alphanumeric_and_dash_underscore_dot",
        "aZ0-_.%20",
    ),
    (
        "prefilled_url_with_newline_in_title",
        "https://github.com/owner/repo/issues/new?title=line1%0Aline2&body=body",
    ),
    ("comment_payload_format", {"body": "text"}),
    ("tool_identity_methods", "p@"),
    ("assemble_body_strips_message", "hello\n\n---\ndiag"),
]


def verify_diagnostics_bearing_issue_surface_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    c0 = percent_encode_query("& #=\n")
    checks.append(
        {
            "name": "percent_encode_query_reserved_characters_uppercase_hex",
            "passed": c0 == "%26%20%23%3D%0A",
        }
    )

    c1 = percent_encode_query("~")
    checks.append(
        {"name": "percent_encode_query_tilde_is_unreserved", "passed": c1 == "~"}
    )

    c2 = percent_encode_query("€")
    checks.append(
        {
            "name": "percent_encode_query_multibyte_utf8",
            "passed": c2 == "%E2%82%AC",
        }
    )

    c3 = assemble_body(None, "diag")
    checks.append(
        {
            "name": "assemble_body_none_message_yields_diagnostics_alone",
            "passed": c3 == "diag",
        }
    )

    c4 = assemble_body("   \n ", "diag")
    checks.append(
        {
            "name": "assemble_body_blank_whitespace_message_yields_diagnostics_alone",
            "passed": c4 == "diag",
        }
    )

    c5 = prefilled_url("owner/repo", "T & T", "B # B", [])
    checks.append(
        {
            "name": "prefilled_url_percent_encodes_title_and_body",
            "passed": c5
            == "https://github.com/owner/repo/issues/new?title=T%20%26%20T&body=B%20%23%20B",
        }
    )

    c6 = prefilled_url("owner/repo", "T", "B", ["a", "b"])
    checks.append(
        {
            "name": "prefilled_url_joins_labels_with_comma",
            "passed": c6
            == "https://github.com/owner/repo/issues/new?title=T&body=B&labels=a%2Cb",
        }
    )

    c7 = percent_encode_query("aZ0-_. ")
    checks.append(
        {
            "name": "percent_encode_query_unreserved_alphanumeric_and_dash_underscore_dot",
            "passed": c7 == "aZ0-_.%20",
        }
    )

    c8 = prefilled_url("owner/repo", "line1\nline2", "body", [])
    checks.append(
        {
            "name": "prefilled_url_with_newline_in_title",
            "passed": c8
            == "https://github.com/owner/repo/issues/new?title=line1%0Aline2&body=body",
        }
    )

    c9 = comment_payload("text")
    checks.append({"name": "comment_payload_format", "passed": c9 == {"body": "text"}})

    tool = ToolInfo("p", "r", "target-triple", "1.0", "sha", "time")
    c10 = tool.tag_prefix()
    checks.append({"name": "tool_identity_methods", "passed": c10 == "p@"})

    c11 = assemble_body("  hello  ", "diag")
    checks.append(
        {"name": "assemble_body_strips_message", "passed": c11 == "hello\n\n---\ndiag"}
    )

    return {
        "case_id": "diagnostics-bearing-issue-surface-security",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
