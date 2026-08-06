from __future__ import annotations

from cli_std.application.upgrade import CheckReport, UpgradePlan, plan_check, plan_upgrade
from cli_std.domain.tool_identity import ToolInfo
from cli_std.domain.version import (
    Action,
    Version,
    decide_action,
    next_command_after_check,
    parse_version,
    select_version,
)

MINIMUM_CHECKS = 14

SELF_UPDATE_FROM_RELEASE_ASSETS_BEHAVIOR_MATRIX = [
    ("tag_prefix_derivation", "mytool@"),
    ("asset_name_and_inner_binary_path_derivations", ("mytool-x86_64-mac.tar.gz", "mytool-x86_64-mac/mytool")),
    ("parse_version_positive_core_and_prerelease", (1, 2, 3, "alpha-beta", "1.2.3-alpha-beta")),
    ("select_version_highest_stable", ("mytool@1.5.0", (1, 5, 0, ""))),
    ("select_version_bare_pin", ("mytool@1.5.0", (1, 5, 0, ""))),
    ("select_version_prefixed_pin", ("mytool@1.5.0", (1, 5, 0, ""))),
    ("select_version_pin_does_not_prefix_match", None),
    ("decide_action_same_version_unforced", "up-to-date"),
    ("decide_action_same_version_forced", "install"),
    ("decide_action_downgrade_unforced", "install"),
    ("next_command_after_check_newer", "mytool upgrade"),
    ("next_command_after_check_same", "done"),
    ("plan_upgrade_unforced_same_version", {"action": "up-to-date", "tag": "mytool@1.0.0", "version": (1, 0, 0, ""), "asset_name": "mytool-x86_64-mac.tar.gz", "inner_binary_path": "mytool-x86_64-mac/mytool"}),
    ("plan_check_same_and_newer_versions", ({"current": (1, 0, 0, ""), "selected": (1, 0, 0, ""), "action": "up-to-date", "next_command": "done"}, {"current": (1, 0, 0, ""), "selected": (1, 5, 0, ""), "action": "install", "next_command": "mytool upgrade"})),
]


def verify_self_update_from_release_assets_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    tool = ToolInfo(
        project="mytool",
        repo="owner/mytool",
        target="x86_64-mac",
        version="1.0.0",
        git_sha="abc1234",
        built_at="2026-01-01",
    )

    c0 = tool.tag_prefix()
    checks.append({"name": "tag_prefix_derivation", "passed": c0 == "mytool@"})

    c1_a = tool.asset_name()
    c1_b = tool.inner_binary_path()
    checks.append({"name": "asset_name_and_inner_binary_path_derivations", "passed": (c1_a, c1_b) == ("mytool-x86_64-mac.tar.gz", "mytool-x86_64-mac/mytool")})

    res_pv = parse_version("1.2.3-alpha-beta")
    c2 = (res_pv.major, res_pv.minor, res_pv.patch, res_pv.pre, res_pv.canonical()) if isinstance(res_pv, Version) else None
    checks.append({"name": "parse_version_positive_core_and_prerelease", "passed": c2 == (1, 2, 3, "alpha-beta", "1.2.3-alpha-beta")})

    tags = ["mytool@1.0.0", "mytool@2.0.0-beta", "mytool@1.5.0", "mytool@1.5.0", "otherx@9.0.0"]
    res3 = select_version(tags, "mytool@", None)
    c3 = (res3[0], (res3[1].major, res3[1].minor, res3[1].patch, res3[1].pre)) if res3 else None
    checks.append({"name": "select_version_highest_stable", "passed": c3 == ("mytool@1.5.0", (1, 5, 0, ""))})

    res4 = select_version(["mytool@1.5.0"], "mytool@", "1.5.0")
    c4 = (res4[0], (res4[1].major, res4[1].minor, res4[1].patch, res4[1].pre)) if res4 else None
    checks.append({"name": "select_version_bare_pin", "passed": c4 == ("mytool@1.5.0", (1, 5, 0, ""))})

    res5 = select_version(["mytool@1.5.0"], "mytool@", "mytool@1.5.0")
    c5 = (res5[0], (res5[1].major, res5[1].minor, res5[1].patch, res5[1].pre)) if res5 else None
    checks.append({"name": "select_version_prefixed_pin", "passed": c5 == ("mytool@1.5.0", (1, 5, 0, ""))})

    res6 = select_version(["mytool@1.5.0"], "mytool@", "1.5")
    c6 = (res6[0], (res6[1].major, res6[1].minor, res6[1].patch, res6[1].pre)) if res6 else None
    checks.append({"name": "select_version_pin_does_not_prefix_match", "passed": c6 is None})

    act7 = decide_action(Version(1, 0, 0, ""), Version(1, 0, 0, ""), False)
    c7 = act7.value if isinstance(act7, Action) else None
    checks.append({"name": "decide_action_same_version_unforced", "passed": c7 == "up-to-date"})

    act8 = decide_action(Version(1, 0, 0, ""), Version(1, 0, 0, ""), True)
    c8 = act8.value if isinstance(act8, Action) else None
    checks.append({"name": "decide_action_same_version_forced", "passed": c8 == "install"})

    act9 = decide_action(Version(1, 5, 0, ""), Version(1, 0, 0, ""), False)
    c9 = act9.value if isinstance(act9, Action) else None
    checks.append({"name": "decide_action_downgrade_unforced", "passed": c9 == "install"})

    c10 = next_command_after_check("mytool", Version(1, 0, 0, ""), Version(1, 5, 0, ""))
    checks.append({"name": "next_command_after_check_newer", "passed": c10 == "mytool upgrade"})

    c11 = next_command_after_check("mytool", Version(1, 0, 0, ""), Version(1, 0, 0, ""))
    checks.append({"name": "next_command_after_check_same", "passed": c11 == "done"})

    res12 = plan_upgrade(tool, ["mytool@1.0.0"], None, False)
    c12 = (
        {
            "action": res12.action.value,
            "tag": res12.tag,
            "version": (res12.version.major, res12.version.minor, res12.version.patch, res12.version.pre),
            "asset_name": res12.asset_name,
            "inner_binary_path": res12.inner_binary_path,
        }
        if isinstance(res12, UpgradePlan)
        else None
    )
    expected_unforced_same_plan = {
        "action": "up-to-date",
        "tag": "mytool@1.0.0",
        "version": (1, 0, 0, ""),
        "asset_name": "mytool-x86_64-mac.tar.gz",
        "inner_binary_path": "mytool-x86_64-mac/mytool",
    }
    checks.append({"name": "plan_upgrade_unforced_same_version", "passed": c12 == expected_unforced_same_plan})

    res13_same = plan_check(tool, ["mytool@1.0.0"], None)
    dict13_same = (
        {
            "current": (res13_same.current.major, res13_same.current.minor, res13_same.current.patch, res13_same.current.pre),
            "selected": (res13_same.selected.major, res13_same.selected.minor, res13_same.selected.patch, res13_same.selected.pre),
            "action": res13_same.action.value,
            "next_command": res13_same.next_command,
        }
        if isinstance(res13_same, CheckReport)
        else None
    )
    res13_newer = plan_check(tool, ["mytool@1.5.0"], None)
    dict13_newer = (
        {
            "current": (res13_newer.current.major, res13_newer.current.minor, res13_newer.current.patch, res13_newer.current.pre),
            "selected": (res13_newer.selected.major, res13_newer.selected.minor, res13_newer.selected.patch, res13_newer.selected.pre),
            "action": res13_newer.action.value,
            "next_command": res13_newer.next_command,
        }
        if isinstance(res13_newer, CheckReport)
        else None
    )
    expected_composite_check = (
        {"current": (1, 0, 0, ""), "selected": (1, 0, 0, ""), "action": "up-to-date", "next_command": "done"},
        {"current": (1, 0, 0, ""), "selected": (1, 5, 0, ""), "action": "install", "next_command": "mytool upgrade"},
    )
    checks.append({"name": "plan_check_same_and_newer_versions", "passed": (dict13_same, dict13_newer) == expected_composite_check})

    return {
        "case_id": "self-update-from-release-assets-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
