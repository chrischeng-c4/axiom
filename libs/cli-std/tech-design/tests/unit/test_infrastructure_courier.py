from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from cli_std.domain.errors import MalformedRepo
from cli_std.infrastructure.courier import (
    bearer_header,
    courier_comment_url,
    courier_create_url,
    courier_search_url,
    courier_view_url,
    github_search_url,
    github_view_url,
    split_repo,
)


class TestInfrastructureCourier(unittest.TestCase):
    def test_courier_view_url_slash_normalization(self) -> None:
        url = courier_view_url("https://c.internal/", "o", "n", 7)
        self.assertIn("https://c.internal/v1/issues/o/n/7", url)
        self.assertNotIn("//v1", url)

    def test_courier_urls_distinctness(self) -> None:
        base = "https://courier.internal"
        u_search = courier_search_url(base, "o", "n", "open", "q", 10)
        u_view = courier_view_url(base, "o", "n", 1)
        u_create = courier_create_url(base, "o", "n")
        u_comment = courier_comment_url(base, "o", "n", 1)

        urls = {u_search, u_view, u_create, u_comment}
        self.assertEqual(len(urls), 4)
        self.assertTrue(u_comment.endswith("/comments"))

    def test_search_urls_parameter_names(self) -> None:
        c_url = courier_search_url("https://c.internal", "o", "n", "open", "a b", 5)
        self.assertIn("state=open", c_url)
        self.assertIn("q=a%20b", c_url)
        self.assertIn("limit=5", c_url)

        gh_url = github_search_url("a b", 5)
        self.assertIn("per_page=5", gh_url)
        self.assertNotIn("limit=", gh_url)

    def test_github_view_url_repo_segment(self) -> None:
        url = github_view_url("owner/name", 7)
        self.assertIn("/repos/owner/name/issues/7", url)

    def test_split_repo_cases(self) -> None:
        self.assertEqual(split_repo("a/b/c"), ("a", "b/c"))
        self.assertEqual(split_repo("abc"), MalformedRepo("abc"))
        self.assertEqual(split_repo("/b"), MalformedRepo("/b"))
        self.assertEqual(split_repo("a/"), MalformedRepo("a/"))

    def test_bearer_header_formatting(self) -> None:
        self.assertEqual(bearer_header("t"), ("Authorization", "Bearer t"))

    def test_courier_create_url_formatting(self) -> None:
        url = courier_create_url("https://courier.io/", "myorg", "myrepo")
        self.assertEqual(url, "https://courier.io/v1/issues/myorg/myrepo")

    def test_courier_comment_url_formatting(self) -> None:
        url = courier_comment_url("https://courier.io", "myorg", "myrepo", 42)
        self.assertEqual(url, "https://courier.io/v1/issues/myorg/myrepo/42/comments")


if __name__ == "__main__":
    unittest.main()
