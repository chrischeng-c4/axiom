from __future__ import annotations

from cli_std.application.issue_routing import (
    BrowserFallback,
    plan_comment,
    plan_create,
    plan_search,
    plan_view,
)
from cli_std.domain.errors import MalformedRepo
from cli_std.domain.tool_identity import ToolInfo
from cli_std.infrastructure.courier import (
    bearer_header,
    courier_comment_url,
    courier_search_url,
    github_search_url,
    split_repo,
)

MINIMUM_CHECKS = 13

COURIER_PROXY_ROUTING_SECURITY_MATRIX = [
    ("split_repo_no_slash_returns_malformed", "noslash"),
    ("split_repo_empty_owner_returns_malformed", "/repo"),
    ("split_repo_empty_name_returns_malformed", "owner/"),
    ("split_repo_first_slash", ("a", "b/c")),
    ("plan_search_refuses_malformed_repo", "bad"),
    ("courier_search_url_encodes_state_and_query", "http://c/v1/issues/o/n?state=open%26x&q=tag%26y&limit=5"),
    ("courier_comment_url_targets_comments_collection", "http://c/v1/issues/o/n/12/comments"),
    ("bearer_header_authorization_name_and_scheme", ("Authorization", "Bearer tok")),
    ("plan_create_unauthenticated_returns_browser_fallback", "https://github.com/o/r/issues/new?title=Title&body=Body&labels=app%3Amytool%2Ctype%3Areport"),
    ("plan_comment_unauthenticated_returns_browser_fallback", "https://github.com/o/r/issues/5"),
    ("github_search_url_encodes_query_and_includes_per_page", "https://api.github.com/search/issues?q=q%20%26%20x&per_page=20"),
    ("plan_view_refuses_malformed_repo", "bad"),
    ("plan_create_refuses_malformed_repo", "bad"),
]


def verify_courier_proxy_routing_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    tool = ToolInfo("mytool", "o/r", "target", "1.0", "sha", "time")

    res0 = split_repo("noslash")
    c0 = res0.repo if isinstance(res0, MalformedRepo) else res0
    checks.append({"name": "split_repo_no_slash_returns_malformed", "passed": c0 == "noslash"})

    res1 = split_repo("/repo")
    c1 = res1.repo if isinstance(res1, MalformedRepo) else res1
    checks.append({"name": "split_repo_empty_owner_returns_malformed", "passed": c1 == "/repo"})

    res2 = split_repo("owner/")
    c2 = res2.repo if isinstance(res2, MalformedRepo) else res2
    checks.append({"name": "split_repo_empty_name_returns_malformed", "passed": c2 == "owner/"})

    c3 = split_repo("a/b/c")
    checks.append({"name": "split_repo_first_slash", "passed": c3 == ("a", "b/c")})

    res4 = plan_search(tool, "bad", None, None, None, "open", None, 10)
    c4 = res4.repo if isinstance(res4, MalformedRepo) else res4
    checks.append({"name": "plan_search_refuses_malformed_repo", "passed": c4 == "bad"})

    c5 = courier_search_url("http://c", "o", "n", "open&x", "tag&y", 5)
    checks.append({"name": "courier_search_url_encodes_state_and_query", "passed": c5 == "http://c/v1/issues/o/n?state=open%26x&q=tag%26y&limit=5"})

    c6 = courier_comment_url("http://c", "o", "n", 12)
    checks.append({"name": "courier_comment_url_targets_comments_collection", "passed": c6 == "http://c/v1/issues/o/n/12/comments"})

    c7 = bearer_header("tok")
    checks.append({"name": "bearer_header_authorization_name_and_scheme", "passed": c7 == ("Authorization", "Bearer tok")})

    res8 = plan_create(tool, "o/r", None, None, None, "Title", "Body", [])
    c8 = res8.url if isinstance(res8, BrowserFallback) else res8
    expected_fallback_create = "https://github.com/o/r/issues/new?title=Title&body=Body&labels=app%3Amytool%2Ctype%3Areport"
    checks.append({"name": "plan_create_unauthenticated_returns_browser_fallback", "passed": c8 == expected_fallback_create})

    res9 = plan_comment(tool, "o/r", None, None, None, 5, "msg")
    c9 = res9.url if isinstance(res9, BrowserFallback) else res9
    expected_fallback_comment = "https://github.com/o/r/issues/5"
    checks.append({"name": "plan_comment_unauthenticated_returns_browser_fallback", "passed": c9 == expected_fallback_comment})

    c10 = github_search_url("q & x", 20)
    checks.append({"name": "github_search_url_encodes_query_and_includes_per_page", "passed": c10 == "https://api.github.com/search/issues?q=q%20%26%20x&per_page=20"})

    res11 = plan_view(tool, "bad", None, None, None, 1)
    c11 = res11.repo if isinstance(res11, MalformedRepo) else res11
    checks.append({"name": "plan_view_refuses_malformed_repo", "passed": c11 == "bad"})

    res12 = plan_create(tool, "bad", None, None, None, "T", "B", [])
    c12 = res12.repo if isinstance(res12, MalformedRepo) else res12
    checks.append({"name": "plan_create_refuses_malformed_repo", "passed": c12 == "bad"})

    return {
        "case_id": "courier-proxy-routing-security",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
