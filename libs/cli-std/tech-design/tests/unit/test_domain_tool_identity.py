from __future__ import annotations

import sys
import unittest
from dataclasses import FrozenInstanceError
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from cli_std.domain.tool_identity import ToolInfo


class TestDomainToolIdentity(unittest.TestCase):
    def test_tool_identity_asset_name_and_inner_path(self) -> None:
        info = ToolInfo(
            project="lumen",
            repo="chrischeng-c4/axiom",
            target="aarch64-apple-darwin",
            version="1.0.0",
            git_sha="abc1234",
            built_at="2026-08-04T12:00:00Z",
        )
        self.assertEqual(info.asset_name(), "lumen-aarch64-apple-darwin.tar.gz")
        self.assertEqual(
            info.inner_binary_path(), "lumen-aarch64-apple-darwin/lumen"
        )

    def test_tool_identity_issue_label_and_tag_prefix(self) -> None:
        info = ToolInfo(
            project="lumen",
            repo="chrischeng-c4/axiom",
            target="aarch64-apple-darwin",
            version="1.0.0",
            git_sha="abc1234",
            built_at="2026-08-04T12:00:00Z",
        )
        self.assertEqual(info.issue_label(), "app:lumen")
        self.assertEqual(info.tag_prefix(), "lumen@")
        self.assertNotIn("aarch64-apple-darwin", info.issue_label())
        self.assertNotIn("aarch64-apple-darwin", info.tag_prefix())

    def test_tool_identity_equality_and_provenance(self) -> None:
        t1 = ToolInfo(
            project="lumen",
            repo="chrischeng-c4/axiom",
            target="aarch64-apple-darwin",
            version="1.0.0",
            git_sha="sha1",
            built_at="2026-08-04T12:00:00Z",
        )
        t2 = ToolInfo(
            project="lumen",
            repo="chrischeng-c4/axiom",
            target="aarch64-apple-darwin",
            version="1.0.0",
            git_sha="sha2",
            built_at="2026-08-04T12:00:00Z",
        )
        self.assertNotEqual(t1, t2)
        self.assertEqual(t1.issue_label(), t2.issue_label())
        self.assertEqual(t1.tag_prefix(), t2.tag_prefix())
        self.assertEqual(t1.asset_name(), t2.asset_name())
        self.assertEqual(t1.inner_binary_path(), t2.inner_binary_path())

    def test_tool_identity_frozen(self) -> None:
        info = ToolInfo(
            project="lumen",
            repo="chrischeng-c4/axiom",
            target="x86_64-unknown-linux-gnu",
            version="1.0.0",
            git_sha="abc1234",
            built_at="2026-08-04T12:00:00Z",
        )
        with self.assertRaises(FrozenInstanceError):
            info.project = "other"  # type: ignore[misc]

    def test_tool_identity_fields_access(self) -> None:
        info = ToolInfo(
            project="lumen",
            repo="owner/repo",
            target="target-triple",
            version="2.1.0",
            git_sha="deadbeef",
            built_at="2026-01-01",
        )
        self.assertEqual(info.project, "lumen")
        self.assertEqual(info.repo, "owner/repo")
        self.assertEqual(info.target, "target-triple")
        self.assertEqual(info.version, "2.1.0")
        self.assertEqual(info.git_sha, "deadbeef")
        self.assertEqual(info.built_at, "2026-01-01")

    def test_tool_identity_different_targets(self) -> None:
        info_mac = ToolInfo("cli", "r", "aarch64-apple-darwin", "1.0", "s", "b")
        info_linux = ToolInfo("cli", "r", "x86_64-unknown-linux-gnu", "1.0", "s", "b")
        self.assertEqual(info_mac.asset_name(), "cli-aarch64-apple-darwin.tar.gz")
        self.assertEqual(info_linux.asset_name(), "cli-x86_64-unknown-linux-gnu.tar.gz")

    def test_tool_identity_repr(self) -> None:
        info = ToolInfo("proj", "repo", "target", "1.0.0", "sha", "now")
        self.assertIn("proj", repr(info))
        self.assertIn("target", repr(info))

    def test_tool_identity_hash_equality(self) -> None:
        i1 = ToolInfo("p", "r", "t", "v", "s", "b")
        i2 = ToolInfo("p", "r", "t", "v", "s", "b")
        self.assertEqual(hash(i1), hash(i2))


if __name__ == "__main__":
    unittest.main()
