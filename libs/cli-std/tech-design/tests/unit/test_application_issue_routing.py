from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from cli_std.application.issue_routing import (
    GITHUB_ACCEPT,
    BrowserFallback,
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
from cli_std.domain.errors import MalformedRepo
from cli_std.domain.tool_identity import ToolInfo


class TestApplicationIssueRouting(unittest.TestCase):
    def setUp(self) -> None:
        self.tool = ToolInfo(
            project="lumen",
            repo="chrischeng-c4/axiom",
            target="aarch64-apple-darwin",
            version="1.0.0",
            git_sha="abc1234",
            built_at="2026-08-04T12:00:00Z",
        )

    def test_choose_route_url_precedence(self) -> None:
        self.assertEqual(choose_route("https://c"), Route.COURIER)
        self.assertEqual(choose_route(None), Route.DIRECT)

    def test_courier_headers_no_accept(self) -> None:
        self.assertEqual(courier_headers(None), ())
        headers_tok = courier_headers("t")
        self.assertNotIn(GITHUB_ACCEPT, headers_tok)
        self.assertEqual(headers_tok, (("Authorization", "Bearer t"),))

    def test_github_headers_accept_and_bearer(self) -> None:
        self.assertEqual(github_headers(None), (GITHUB_ACCEPT,))
        self.assertEqual(
            github_headers("t"), (GITHUB_ACCEPT, ("Authorization", "Bearer t"))
        )

    def test_build_search_query_qualifiers_by_route(self) -> None:
        q_courier = build_search_query(
            self.tool, "chrischeng-c4/axiom", Route.COURIER, "open", None
        )
        self.assertNotIn("repo:", q_courier)
        self.assertNotIn("is:issue", q_courier)
        self.assertNotIn("state:", q_courier)
        self.assertIn('label:"app:lumen"', q_courier)

        q_direct = build_search_query(
            self.tool, "chrischeng-c4/axiom", Route.DIRECT, "open", None
        )
        self.assertIn("repo:chrischeng-c4/axiom", q_direct)
        self.assertIn("is:issue", q_direct)
        self.assertIn("state:open", q_direct)
        self.assertIn('label:"app:lumen"', q_direct)

    def test_build_search_query_state_all_omitted(self) -> None:
        q_all = build_search_query(
            self.tool, "chrischeng-c4/axiom", Route.DIRECT, "all", None
        )
        self.assertNotIn("state:", q_all)

    def test_build_search_query_free_text_stripping(self) -> None:
        q_blank = build_search_query(
            self.tool, "o/n", Route.DIRECT, "open", "   "
        )
        q_none = build_search_query(
            self.tool, "o/n", Route.DIRECT, "open", None
        )
        self.assertEqual(q_blank, q_none)

        q_text = build_search_query(
            self.tool, "o/n", Route.DIRECT, "open", "  hi  "
        )
        self.assertTrue(q_text.endswith(" hi"))

    def test_plan_search_courier_url_and_query_state(self) -> None:
        res = plan_search(
            self.tool,
            "chrischeng-c4/axiom",
            "https://c",
            "ctok",
            None,
            "open",
            "query text",
            10,
        )
        self.assertIsInstance(res, RequestPlan)
        if isinstance(res, RequestPlan):
            self.assertIn("state=open", res.url)

    def test_plan_search_malformed_repo_both_routes(self) -> None:
        res_courier = plan_search(
            self.tool, "abc", "https://c", None, None, "open", None, 5
        )
        self.assertIsInstance(res_courier, MalformedRepo)

        res_direct = plan_search(
            self.tool, "abc", None, None, "gtok", "open", None, 5
        )
        self.assertIsInstance(res_direct, MalformedRepo)

    def test_plan_create_courier_no_github_token(self) -> None:
        res = plan_create(
            self.tool,
            "chrischeng-c4/axiom",
            "https://c",
            "ctok",
            None,
            "Title",
            "Body",
            [],
        )
        self.assertIsInstance(res, RequestPlan)
        if isinstance(res, RequestPlan):
            self.assertEqual(res.route, Route.COURIER)

    def test_plan_create_direct_no_github_token_browser_fallback(self) -> None:
        res = plan_create(
            self.tool,
            "chrischeng-c4/axiom",
            None,
            None,
            None,
            "Title",
            "Body",
            [],
        )
        self.assertIsInstance(res, BrowserFallback)
        if isinstance(res, BrowserFallback):
            self.assertIn("type%3Areport", res.url)
            self.assertIn("app%3Alumen", res.url)

    def test_plan_create_report_labels_application(self) -> None:
        res_empty = plan_create(
            self.tool,
            "chrischeng-c4/axiom",
            None,
            None,
            "gtok",
            "T",
            "B",
            [],
        )
        self.assertIsInstance(res_empty, RequestPlan)
        if isinstance(res_empty, RequestPlan) and res_empty.body:
            self.assertIn("labels", res_empty.body)
            self.assertIn("type:report", res_empty.body["labels"])

        res_dup = plan_create(
            self.tool,
            "chrischeng-c4/axiom",
            None,
            None,
            "gtok",
            "T",
            "B",
            ["type:report"],
        )
        if isinstance(res_dup, RequestPlan) and res_dup.body:
            labels_list = res_dup.body["labels"]
            self.assertIsInstance(labels_list, list)
            if isinstance(labels_list, list):
                self.assertEqual(labels_list.count("type:report"), 1)

    def test_plan_comment_tuple_lengths(self) -> None:
        c_res = plan_comment(
            self.tool,
            "chrischeng-c4/axiom",
            "https://c",
            "ctok",
            None,
            42,
            "msg",
        )
        self.assertIsInstance(c_res, tuple)
        if isinstance(c_res, tuple):
            self.assertEqual(len(c_res), 1)

        d_res = plan_comment(
            self.tool,
            "chrischeng-c4/axiom",
            None,
            None,
            "gtok",
            42,
            "msg",
        )
        self.assertIsInstance(d_res, tuple)
        if isinstance(d_res, tuple):
            self.assertEqual(len(d_res), 2)

    def test_plan_comment_direct_two_tuple_details(self) -> None:
        d_res = plan_comment(
            self.tool,
            "chrischeng-c4/axiom",
            None,
            None,
            "gtok",
            42,
            "msg",
        )
        self.assertIsInstance(d_res, tuple)
        if isinstance(d_res, tuple) and len(d_res) == 2:
            patch_req, post_req = d_res
            self.assertEqual(patch_req.method, "PATCH")
            self.assertEqual(patch_req.body, {"state": "open"})

            self.assertEqual(post_req.method, "POST")
            self.assertIsNotNone(post_req.body)
            if post_req.body:
                self.assertEqual(set(post_req.body.keys()), {"body"})

    def test_plan_view_routes(self) -> None:
        v_courier = plan_view(
            self.tool, "owner/repo", "https://c", None, None, 7
        )
        self.assertIsInstance(v_courier, RequestPlan)
        if isinstance(v_courier, RequestPlan):
            self.assertEqual(v_courier.route, Route.COURIER)

        v_direct = plan_view(self.tool, "owner/repo", None, None, "gtok", 7)
        self.assertIsInstance(v_direct, RequestPlan)
        if isinstance(v_direct, RequestPlan):
            self.assertEqual(v_direct.route, Route.DIRECT)


if __name__ == "__main__":
    unittest.main()
