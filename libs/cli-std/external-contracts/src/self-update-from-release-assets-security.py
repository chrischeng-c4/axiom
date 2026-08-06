from __future__ import annotations

from cli_std.application.errors import UnreadableCurrentVersion
from cli_std.application.upgrade import (
    UpgradePlan,
    accept_asset,
    locate_inner_binary,
    plan_upgrade,
)
from cli_std.domain.errors import DigestMismatch, MissingInnerBinary
from cli_std.domain.tool_identity import ToolInfo
from cli_std.domain.version import Action, Version, compare_versions, parse_version

MINIMUM_CHECKS = 13

SELF_UPDATE_FROM_RELEASE_ASSETS_SECURITY_MATRIX = [
    ("parse_version_refuses_surrounding_whitespace", None),
    ("parse_version_refuses_interior_space", None),
    ("parse_version_refuses_bare_trailing_hyphen", None),
    ("parse_version_refuses_four_part_core", None),
    ("parse_version_refuses_non_ascii_digits", None),
    ("parse_version_refuses_leading_zero", None),
    ("compare_versions_minor_and_patch", True),
    ("compare_versions_release_outranks_prerelease", 1),
    ("compare_versions_prerelease_ordering", -1),
    ("accept_asset_matches_trimmed_case_insensitive_sha256", None),
    ("accept_asset_mismatch_fields_order", {"expected": "bad", "actual": "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"}),
    ("locate_inner_binary_missing_returns_error", ("missing", "mytool-target/mytool")),
    ("plan_upgrade_unreadable_version_precedence", "invalid"),
]


def verify_self_update_from_release_assets_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    res0 = parse_version("1.0.0-alpha ")
    c0 = (res0.major, res0.minor, res0.patch, res0.pre) if isinstance(res0, Version) else None
    checks.append({"name": "parse_version_refuses_surrounding_whitespace", "passed": c0 is None})

    res1 = parse_version("1.0.0-alpha beta")
    c1 = (res1.major, res1.minor, res1.patch, res1.pre) if isinstance(res1, Version) else None
    checks.append({"name": "parse_version_refuses_interior_space", "passed": c1 is None})

    res2 = parse_version("1.0.0-")
    c2 = (res2.major, res2.minor, res2.patch, res2.pre) if isinstance(res2, Version) else None
    checks.append({"name": "parse_version_refuses_bare_trailing_hyphen", "passed": c2 is None})

    res3 = parse_version("1.0.0.0")
    c3 = (res3.major, res3.minor, res3.patch, res3.pre) if isinstance(res3, Version) else None
    checks.append({"name": "parse_version_refuses_four_part_core", "passed": c3 is None})

    res_non_ascii = parse_version("١.0.0")
    c_non_ascii = (res_non_ascii.major, res_non_ascii.minor, res_non_ascii.patch, res_non_ascii.pre) if isinstance(res_non_ascii, Version) else None
    checks.append({"name": "parse_version_refuses_non_ascii_digits", "passed": c_non_ascii is None})

    res4 = parse_version("01.0.0")
    c4 = (res4.major, res4.minor, res4.patch, res4.pre) if isinstance(res4, Version) else None
    checks.append({"name": "parse_version_refuses_leading_zero", "passed": c4 is None})

    c6 = (
        compare_versions(Version(1, 2, 0, ""), Version(1, 1, 0, "")) == 1
        and compare_versions(Version(1, 0, 2, ""), Version(1, 0, 1, "")) == 1
    )
    checks.append({"name": "compare_versions_minor_and_patch", "passed": c6 == True})

    c7 = compare_versions(Version(1, 0, 0, ""), Version(1, 0, 0, "alpha"))
    checks.append({"name": "compare_versions_release_outranks_prerelease", "passed": c7 == 1})

    c8 = compare_versions(Version(1, 0, 0, "alpha"), Version(1, 0, 0, "beta"))
    checks.append({"name": "compare_versions_prerelease_ordering", "passed": c8 == -1})

    payload = b"hello"
    sha = " 2CF24DBA5FB0A30E26E83B2AC5B9E29E1B161E5C1FA7425E73043362938B9824 "
    res9 = accept_asset(payload, sha)
    c9 = {"expected": res9.expected, "actual": res9.actual} if isinstance(res9, DigestMismatch) else None
    checks.append({"name": "accept_asset_matches_trimmed_case_insensitive_sha256", "passed": c9 is None})

    res10 = accept_asset(payload, "bad")
    c10 = {"expected": res10.expected, "actual": res10.actual} if isinstance(res10, DigestMismatch) else None
    expected_mismatch = {"expected": "bad", "actual": "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"}
    checks.append({"name": "accept_asset_mismatch_fields_order", "passed": c10 == expected_mismatch})

    plan = UpgradePlan(
        action=Action.INSTALL,
        tag="t",
        version=Version(1, 0, 0, ""),
        asset_name="a",
        inner_binary_path="mytool-target/mytool",
    )
    res11 = locate_inner_binary(["mytool-target/other"], plan)
    c11 = ("missing", res11.inner_path) if isinstance(res11, MissingInnerBinary) else (("found", res11) if isinstance(res11, str) else None)
    checks.append({"name": "locate_inner_binary_missing_returns_error", "passed": c11 == ("missing", "mytool-target/mytool")})

    class ExplodingTag(str):
        def startswith(self, prefix: object, start: int | None = None, end: int | None = None) -> bool:
            raise RuntimeError("should not inspect releases when installed version is invalid")

    tool_bad = ToolInfo("mytool", "repo", "target", "invalid", "sha", "time")
    res12 = plan_upgrade(tool_bad, (ExplodingTag("mytool@1.0.0"),), None, False)
    c12 = res12.text if isinstance(res12, UnreadableCurrentVersion) else None
    checks.append({"name": "plan_upgrade_unreadable_version_precedence", "passed": c12 == "invalid"})

    return {
        "case_id": "self-update-from-release-assets-security",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
