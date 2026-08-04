from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from cli_std.application.errors import NoMatchingRelease, UnreadableCurrentVersion
from cli_std.application.upgrade import (
    CheckReport,
    UpgradePlan,
    accept_asset,
    locate_inner_binary,
    plan_check,
    plan_upgrade,
)
from cli_std.domain.errors import DigestMismatch, MissingInnerBinary
from cli_std.domain.tool_identity import ToolInfo
from cli_std.domain.version import Action, Version


class TestApplicationUpgrade(unittest.TestCase):
    def setUp(self) -> None:
        self.tool = ToolInfo(
            project="lumen",
            repo="chrischeng-c4/axiom",
            target="aarch64-apple-darwin",
            version="1.0.0",
            git_sha="abc1234",
            built_at="2026-08-04T12:00:00Z",
        )

    def test_plan_upgrade_unreadable_current_version(self) -> None:
        nightly_tool = ToolInfo(
            project="lumen",
            repo="chrischeng-c4/axiom",
            target="aarch64-apple-darwin",
            version="nightly",
            git_sha="abc1234",
            built_at="2026-08-04T12:00:00Z",
        )
        res = plan_upgrade(nightly_tool, ["lumen@2.0.0"], None, False)
        self.assertIsInstance(res, UnreadableCurrentVersion)
        if isinstance(res, UnreadableCurrentVersion):
            self.assertEqual(res.text, "nightly")

    def test_plan_upgrade_equal_version_up_to_date_plan(self) -> None:
        res = plan_upgrade(self.tool, ["lumen@1.0.0"], None, False)
        self.assertIsInstance(res, UpgradePlan)
        if isinstance(res, UpgradePlan):
            self.assertEqual(res.action, Action.UP_TO_DATE)
            self.assertEqual(res.tag, "lumen@1.0.0")
            self.assertEqual(res.version, Version(1, 0, 0, ""))

    def test_upgrade_plan_asset_name_from_tool(self) -> None:
        res = plan_upgrade(self.tool, ["lumen@2.0.0"], None, False)
        self.assertIsInstance(res, UpgradePlan)
        if isinstance(res, UpgradePlan):
            self.assertEqual(res.asset_name, self.tool.asset_name())
            self.assertEqual(res.inner_binary_path, self.tool.inner_binary_path())
            self.assertNotIn("2.0.0", res.asset_name)

    def test_plan_upgrade_no_matching_release_pin(self) -> None:
        res = plan_upgrade(self.tool, ["lumen@1.0.0"], "9.9.9", False)
        self.assertIsInstance(res, NoMatchingRelease)
        if isinstance(res, NoMatchingRelease):
            self.assertEqual(res.prefix, "lumen@")
            self.assertEqual(res.pin, "9.9.9")

    def test_plan_check_actions_and_next_command(self) -> None:
        c_equal = plan_check(self.tool, ["lumen@1.0.0"], None)
        self.assertIsInstance(c_equal, CheckReport)
        if isinstance(c_equal, CheckReport):
            self.assertEqual(c_equal.action, Action.UP_TO_DATE)
            self.assertEqual(c_equal.next_command, "done")

        c_older = plan_check(self.tool, ["lumen@0.9.0"], None)
        self.assertIsInstance(c_older, CheckReport)
        if isinstance(c_older, CheckReport):
            self.assertEqual(c_older.next_command, "done")

        tool_v0 = ToolInfo(
            project="lumen",
            repo="chrischeng-c4/axiom",
            target="aarch64-apple-darwin",
            version="0.9.0",
            git_sha="abc",
            built_at="2026-01-01",
        )
        c_newer = plan_check(tool_v0, ["lumen@1.0.0"], None)
        self.assertIsInstance(c_newer, CheckReport)
        if isinstance(c_newer, CheckReport):
            self.assertEqual(c_newer.next_command, "lumen upgrade")

    def test_accept_asset_none_expected_digest(self) -> None:
        self.assertIsNone(accept_asset(b"x", None))

    def test_accept_asset_verification_outcomes(self) -> None:
        empty_digest = (
            "E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855"
        )
        self.assertIsNone(accept_asset(b"", empty_digest))

        mismatch = accept_asset(b"", "00" * 32)
        self.assertIsInstance(mismatch, DigestMismatch)

    def test_locate_inner_binary_exact_matching(self) -> None:
        plan = UpgradePlan(
            action=Action.INSTALL,
            tag="lumen@1.0.0",
            version=Version(1, 0, 0, ""),
            asset_name="lumen-aarch64-apple-darwin.tar.gz",
            inner_binary_path="lumen-aarch64-apple-darwin/lumen",
        )

        path = "lumen-aarch64-apple-darwin/lumen"
        self.assertEqual(locate_inner_binary([path], plan), path)

        prefix_path = "x/lumen-aarch64-apple-darwin/lumen"
        res_prefix = locate_inner_binary([prefix_path], plan)
        self.assertIsInstance(res_prefix, MissingInnerBinary)

    def test_plan_upgrade_force_flag_installs_same_version(self) -> None:
        res = plan_upgrade(self.tool, ["lumen@1.0.0"], None, force=True)
        self.assertIsInstance(res, UpgradePlan)
        if isinstance(res, UpgradePlan):
            self.assertEqual(res.action, Action.INSTALL)

    def test_plan_upgrade_newer_version_installs(self) -> None:
        res = plan_upgrade(self.tool, ["lumen@2.0.0"], None, force=False)
        self.assertIsInstance(res, UpgradePlan)
        if isinstance(res, UpgradePlan):
            self.assertEqual(res.action, Action.INSTALL)

    def test_plan_check_unreadable_current_version(self) -> None:
        bad_tool = ToolInfo("p", "r", "t", "bad-ver", "s", "b")
        res = plan_check(bad_tool, ["p@1.0.0"], None)
        self.assertIsInstance(res, UnreadableCurrentVersion)

    def test_locate_inner_binary_missing_returns_error(self) -> None:
        plan = UpgradePlan(
            action=Action.INSTALL,
            tag="t@1.0",
            version=Version(1, 0, 0, ""),
            asset_name="a.tar.gz",
            inner_binary_path="dir/bin",
        )
        res = locate_inner_binary(["other/bin"], plan)
        self.assertEqual(res, MissingInnerBinary("dir/bin"))


if __name__ == "__main__":
    unittest.main()
