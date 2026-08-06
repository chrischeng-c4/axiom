from __future__ import annotations

from cli_std.application.issue_routing import (
    GITHUB_ACCEPT,
    RequestPlan,
    Route,
    build_search_query,
    choose_route,
    courier_headers,
    github_headers,
    plan_comment,
    plan_create,
    plan_search,
    plan_view,
)
from cli_std.domain.tool_identity import ToolInfo
from cli_std.infrastructure.courier import github_search_url

MINIMUM_CHECKS = 13

COURIER_PROXY_ROUTING_BEHAVIOR_MATRIX = [
    ("choose_route_courier_and_direct", ("courier", "direct")),
    ("github_accept_header_constant", ("Accept", "application/vnd.github+json")),
    ("github_headers_order_and_unauthenticated", ((("Accept", "application/vnd.github+json"), ("Authorization", "Bearer gh_tok")), (("Accept", "application/vnd.github+json"),))),
    ("courier_headers_format", (("Authorization", "Bearer c_tok"),)),
    ("build_search_query_courier_omits_repo", 'label:"app:mytool" bug'),
    ("build_search_query_direct_includes_repo", 'repo:o/r is:issue label:"app:mytool" state:open bug'),
    ("build_search_query_direct_state_all_omits_state_filter", ('label:"app:mytool"', 'repo:o/r is:issue label:"app:mytool"')),
    (
        "plan_search_courier_normalizes_trailing_slash",
        {"route": "courier", "method": "GET", "url": "http://courier/v1/issues/o/r?state=open&q=label%3A%22app%3Amytool%22%20bug&limit=10", "headers": (("Authorization", "Bearer c_tok"),), "body": None},
    ),
    (
        "plan_view_courier_url_and_headers",
        {"route": "courier", "method": "GET", "url": "http://courier/v1/issues/o/r/42", "headers": (("Authorization", "Bearer c_tok"),), "body": None},
    ),
    (
        "plan_create_courier_and_direct_post_requests",
        (
            {"route": "courier", "method": "POST", "url": "http://courier/v1/issues/o/r", "headers": (("Authorization", "Bearer c_tok"),), "body": {"title": "T", "body": "B", "labels": ["app:mytool", "type:report"]}},
            {"route": "direct", "method": "POST", "url": "https://api.github.com/repos/o/r/issues", "headers": (("Accept", "application/vnd.github+json"), ("Authorization", "Bearer gh_tok")), "body": {"title": "T", "body": "B", "labels": ["app:mytool", "type:report"]}},
        ),
    ),
    (
        "plan_comment_direct_reopens_then_posts",
        (
            {"route": "direct", "method": "PATCH", "url": "https://api.github.com/repos/o/r/issues/42", "headers": (("Accept", "application/vnd.github+json"), ("Authorization", "Bearer gh_tok")), "body": {"state": "open"}},
            {"route": "direct", "method": "POST", "url": "https://api.github.com/repos/o/r/issues/42/comments", "headers": (("Accept", "application/vnd.github+json"), ("Authorization", "Bearer gh_tok")), "body": {"body": "msg"}},
        ),
    ),
    (
        "plan_comment_courier_single_element_tuple",
        (
            {"route": "courier", "method": "POST", "url": "http://courier/v1/issues/o/r/42/comments", "headers": (("Authorization", "Bearer c_tok"),), "body": {"body": "msg"}},
        ),
    ),
    ("github_search_url_format", "https://api.github.com/search/issues?q=query&per_page=10"),
]


def verify_courier_proxy_routing_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []
    tool = ToolInfo("mytool", "o/r", "target", "1.0", "sha", "time")

    r0_a = choose_route("http://courier")
    r0_b = choose_route(None)
    c0 = (r0_a.value if isinstance(r0_a, Route) else None, r0_b.value if isinstance(r0_b, Route) else None)
    checks.append({"name": "choose_route_courier_and_direct", "passed": c0 == ("courier", "direct")})

    c1 = GITHUB_ACCEPT
    checks.append({"name": "github_accept_header_constant", "passed": c1 == ("Accept", "application/vnd.github+json")})

    c2 = (github_headers("gh_tok"), github_headers(None))
    expected_gh_headers = (
        (("Accept", "application/vnd.github+json"), ("Authorization", "Bearer gh_tok")),
        (("Accept", "application/vnd.github+json"),),
    )
    checks.append({"name": "github_headers_order_and_unauthenticated", "passed": c2 == expected_gh_headers})

    c3 = courier_headers("c_tok")
    checks.append({"name": "courier_headers_format", "passed": c3 == (("Authorization", "Bearer c_tok"),)})

    c4 = build_search_query(tool, "o/r", Route.COURIER, "open", "  bug  ")
    checks.append({"name": "build_search_query_courier_omits_repo", "passed": c4 == 'label:"app:mytool" bug'})

    c5 = build_search_query(tool, "o/r", Route.DIRECT, "open", "bug")
    checks.append({"name": "build_search_query_direct_includes_repo", "passed": c5 == 'repo:o/r is:issue label:"app:mytool" state:open bug'})

    c6_a = build_search_query(tool, "o/r", Route.COURIER, "open", "   ")
    c6_b = build_search_query(tool, "o/r", Route.DIRECT, "all", None)
    checks.append({"name": "build_search_query_direct_state_all_omits_state_filter", "passed": (c6_a, c6_b) == ('label:"app:mytool"', 'repo:o/r is:issue label:"app:mytool"')})

    res7 = plan_search(tool, "o/r", "http://courier/", "c_tok", None, "open", "bug", 10)
    c7 = {"route": res7.route.value, "method": res7.method, "url": res7.url, "headers": res7.headers, "body": res7.body} if isinstance(res7, RequestPlan) else None
    expected_search_plan = {"route": "courier", "method": "GET", "url": "http://courier/v1/issues/o/r?state=open&q=label%3A%22app%3Amytool%22%20bug&limit=10", "headers": (("Authorization", "Bearer c_tok"),), "body": None}
    checks.append({"name": "plan_search_courier_normalizes_trailing_slash", "passed": c7 == expected_search_plan})

    res8 = plan_view(tool, "o/r", "http://courier", "c_tok", None, 42)
    c8 = {"route": res8.route.value, "method": res8.method, "url": res8.url, "headers": res8.headers, "body": res8.body} if isinstance(res8, RequestPlan) else None
    expected_view_plan = {"route": "courier", "method": "GET", "url": "http://courier/v1/issues/o/r/42", "headers": (("Authorization", "Bearer c_tok"),), "body": None}
    checks.append({"name": "plan_view_courier_url_and_headers", "passed": c8 == expected_view_plan})

    res9_c = plan_create(tool, "o/r", "http://courier", "c_tok", None, "T", "B", [])
    dict9_c = {"route": res9_c.route.value, "method": res9_c.method, "url": res9_c.url, "headers": res9_c.headers, "body": res9_c.body} if isinstance(res9_c, RequestPlan) else None
    res9_d = plan_create(tool, "o/r", None, None, "gh_tok", "T", "B", [])
    dict9_d = {"route": res9_d.route.value, "method": res9_d.method, "url": res9_d.url, "headers": res9_d.headers, "body": res9_d.body} if isinstance(res9_d, RequestPlan) else None
    expected_create_plans = (
        {"route": "courier", "method": "POST", "url": "http://courier/v1/issues/o/r", "headers": (("Authorization", "Bearer c_tok"),), "body": {"title": "T", "body": "B", "labels": ["app:mytool", "type:report"]}},
        {"route": "direct", "method": "POST", "url": "https://api.github.com/repos/o/r/issues", "headers": (("Accept", "application/vnd.github+json"), ("Authorization", "Bearer gh_tok")), "body": {"title": "T", "body": "B", "labels": ["app:mytool", "type:report"]}},
    )
    checks.append({"name": "plan_create_courier_and_direct_post_requests", "passed": (dict9_c, dict9_d) == expected_create_plans})

    res10 = plan_comment(tool, "o/r", None, None, "gh_tok", 42, "msg")
    c10 = tuple({"route": r.route.value, "method": r.method, "url": r.url, "headers": r.headers, "body": r.body} for r in res10) if isinstance(res10, tuple) else None
    expected_comment_direct = (
        {"route": "direct", "method": "PATCH", "url": "https://api.github.com/repos/o/r/issues/42", "headers": (("Accept", "application/vnd.github+json"), ("Authorization", "Bearer gh_tok")), "body": {"state": "open"}},
        {"route": "direct", "method": "POST", "url": "https://api.github.com/repos/o/r/issues/42/comments", "headers": (("Accept", "application/vnd.github+json"), ("Authorization", "Bearer gh_tok")), "body": {"body": "msg"}},
    )
    checks.append({"name": "plan_comment_direct_reopens_then_posts", "passed": c10 == expected_comment_direct})

    res11 = plan_comment(tool, "o/r", "http://courier", "c_tok", None, 42, "msg")
    c11 = tuple({"route": r.route.value, "method": r.method, "url": r.url, "headers": r.headers, "body": r.body} for r in res11) if isinstance(res11, tuple) else None
    expected_comment_courier = (
        {"route": "courier", "method": "POST", "url": "http://courier/v1/issues/o/r/42/comments", "headers": (("Authorization", "Bearer c_tok"),), "body": {"body": "msg"}},
    )
    checks.append({"name": "plan_comment_courier_single_element_tuple", "passed": c11 == expected_comment_courier})

    c12 = github_search_url("query", 10)
    checks.append({"name": "github_search_url_format", "passed": c12 == "https://api.github.com/search/issues?q=query&per_page=10"})

    return {
        "case_id": "courier-proxy-routing-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
