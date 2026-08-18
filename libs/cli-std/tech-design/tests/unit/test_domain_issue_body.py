from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from cli_std.domain.issue_body import (
    assemble_body,
    followup_comment_body,
    issue_payload,
    percent_encode_query,
    prefilled_url,
    render_diagnostics,
    report_labels,
    resolve_repo,
)
from cli_std.domain.tool_identity import ToolInfo


class TestDomainIssueBody(unittest.TestCase):
    def setUp(self) -> None:
        self.tool = ToolInfo(
            project="lumen",
            repo="chrischeng-c4/axiom",
            target="aarch64-apple-darwin",
            version="1.0.0",
            git_sha="abc1234",
            built_at="2026-08-04T12:00:00Z",
        )

    def test_percent_encode_query_utf8(self) -> None:
        self.assertEqual(percent_encode_query("a b"), "a%20b")
        self.assertEqual(percent_encode_query("a-b_c.d~e"), "a-b_c.d~e")
        self.assertEqual(percent_encode_query("a/b"), "a%2Fb")
        self.assertEqual(percent_encode_query("é"), "%C3%A9")
        self.assertEqual(
            percent_encode_query("type:report,app:lumen"),
            "type%3Areport%2Capp%3Alumen",
        )

    def test_assemble_body_empty_message(self) -> None:
        diag = "## Diagnostics\nsome data\n"
        res = assemble_body("   ", diag)
        self.assertEqual(res, diag)
        self.assertIs(res, diag)

    def test_assemble_body_with_message(self) -> None:
        diag = "## Diagnostics\nsome data\n"
        res = assemble_body("  msg  ", diag)
        self.assertTrue(res.startswith("msg"))
        self.assertEqual(res.count("\n\n---\n"), 1)
        self.assertIn("msg\n\n---\n" + diag, res)

    def test_render_diagnostics_node_filtering(self) -> None:
        no_node = render_diagnostics(self.tool, "linux", "x86_64", None)
        self.assertNotIn("- node:", no_node)

        with_node = render_diagnostics(self.tool, "linux", "x86_64", "pod-0")
        lines = with_node.strip().split("\n")
        self.assertEqual(lines[-1], "- node: pod-0")

    def test_report_labels_deduplication(self) -> None:
        labels = report_labels(self.tool, ["type:report"])
        self.assertEqual(len(labels), 2)
        self.assertEqual(labels, ("type:report", "app:lumen"))

    def test_report_labels_empty_input(self) -> None:
        labels = report_labels(self.tool, [])
        self.assertEqual(labels, ("app:lumen", "type:report"))

    def test_issue_payload_labels_key_absence(self) -> None:
        payload_empty = issue_payload("title", "body", [])
        self.assertNotIn("labels", payload_empty)

        payload_with = issue_payload("title", "body", ["bug"])
        self.assertIn("labels", payload_with)
        self.assertIsInstance(payload_with["labels"], list)
        self.assertEqual(payload_with["labels"], ["bug"])

    def test_resolve_repo_empty_override(self) -> None:
        self.assertEqual(resolve_repo(self.tool, ""), "")
        self.assertEqual(resolve_repo(self.tool, None), "chrischeng-c4/axiom")
        self.assertEqual(resolve_repo(self.tool, "other/repo"), "other/repo")

    def test_prefilled_url_encoding(self) -> None:
        url = prefilled_url("owner/repo", "Title with space", "Body", [])
        self.assertIn("https://github.com/owner/repo/issues/new", url)
        self.assertIn("title=Title%20with%20space", url)
        self.assertNotIn("&labels=", url)

        url_labels = prefilled_url(
            "owner/repo", "Title", "Body", ["app:lumen", "type:report"]
        )
        self.assertIn("&labels=app%3Alumen%2Ctype%3Areport", url_labels)

    def test_followup_comment_body_default(self) -> None:
        body_none = followup_comment_body(self.tool, None, "mac", "arm64")
        body_blank = followup_comment_body(self.tool, "  ", "mac", "arm64")
        default_sentence = (
            "User-side verification failed after closure; reopening for follow-up."
        )
        self.assertIn(default_sentence, body_none)
        self.assertEqual(body_none, body_blank)


if __name__ == "__main__":
    unittest.main()
