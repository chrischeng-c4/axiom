from __future__ import annotations

from cli_std.infrastructure.credentials import (
    resolve_courier_token,
    resolve_courier_url,
    resolve_github_token,
)

MINIMUM_CHECKS = 10

GITHUB_CREDENTIAL_RESOLUTION_ORDER_SECURITY_MATRIX = [
    ("blank_gh_token_does_not_shadow_github_token", "real_github_token"),
    ("blank_gh_token_and_github_token_fall_through_to_helper", "real_helper_token"),
    ("empty_string_gh_token_falls_through", "real_github_token"),
    ("absent_credentials_everywhere_returns_none", None),
    ("blank_courier_url_returns_none", None),
    ("empty_courier_url_returns_none", None),
    ("absent_courier_url_returns_none", None),
    ("blank_courier_token_returns_none", None),
    ("empty_courier_token_returns_none", None),
    ("absent_courier_token_returns_none", None),
]


def verify_github_credential_resolution_order_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    c0 = resolve_github_token(
        lambda k: "   "
        if k == "GH_TOKEN"
        else ("real_github_token" if k == "GITHUB_TOKEN" else None),
        lambda: None,
    )
    checks.append(
        {
            "name": "blank_gh_token_does_not_shadow_github_token",
            "passed": c0 == "real_github_token",
        }
    )

    c1 = resolve_github_token(
        lambda k: "   ",
        lambda: "real_helper_token",
    )
    checks.append(
        {
            "name": "blank_gh_token_and_github_token_fall_through_to_helper",
            "passed": c1 == "real_helper_token",
        }
    )

    c2 = resolve_github_token(
        lambda k: "" if k == "GH_TOKEN" else "real_github_token",
        lambda: None,
    )
    checks.append(
        {
            "name": "empty_string_gh_token_falls_through",
            "passed": c2 == "real_github_token",
        }
    )

    c3 = resolve_github_token(lambda k: None, lambda: None)
    checks.append(
        {
            "name": "absent_credentials_everywhere_returns_none",
            "passed": c3 is None,
        }
    )

    c4 = resolve_courier_url(lambda k: "   ")
    checks.append({"name": "blank_courier_url_returns_none", "passed": c4 is None})

    c5 = resolve_courier_url(lambda k: "")
    checks.append({"name": "empty_courier_url_returns_none", "passed": c5 is None})

    c6 = resolve_courier_url(lambda k: None)
    checks.append({"name": "absent_courier_url_returns_none", "passed": c6 is None})

    c7 = resolve_courier_token(lambda k: "   ")
    checks.append({"name": "blank_courier_token_returns_none", "passed": c7 is None})

    c8 = resolve_courier_token(lambda k: "")
    checks.append({"name": "empty_courier_token_returns_none", "passed": c8 is None})

    c9 = resolve_courier_token(lambda k: None)
    checks.append({"name": "absent_courier_token_returns_none", "passed": c9 is None})

    return {
        "case_id": "github-credential-resolution-order-security",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
