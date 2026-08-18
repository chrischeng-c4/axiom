from __future__ import annotations

from cli_std.infrastructure.credentials import (
    COURIER_TOKEN_KEY,
    COURIER_URL_KEY,
    GH_TOKEN_KEY,
    GITHUB_TOKEN_KEY,
    resolve_courier_token,
    resolve_courier_url,
    resolve_github_token,
)

MINIMUM_CHECKS = 10

GITHUB_CREDENTIAL_RESOLUTION_ORDER_BEHAVIOR_MATRIX = [
    ("gh_token_key_literal", "GH_TOKEN"),
    ("github_token_key_literal", "GITHUB_TOKEN"),
    ("courier_url_key_literal", "AXIOM_COURIER_URL"),
    ("courier_token_key_literal", "AXIOM_COURIER_TOKEN"),
    ("gh_token_precedes_github_token_and_gh_not_called", "tok_gh"),
    ("github_token_used_when_gh_token_absent", "tok_github"),
    ("gh_helper_consulted_last", "helper_tok"),
    ("resolved_token_trimmed", "padded_tok"),
    ("resolve_courier_url_trimmed", "http://url"),
    ("resolve_courier_token_trimmed", "secret_tok"),
]


def verify_github_credential_resolution_order_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    def forbidden_helper() -> str:
        raise RuntimeError("helper should not be called when environment token present")

    c0 = GH_TOKEN_KEY
    checks.append({"name": "gh_token_key_literal", "passed": c0 == "GH_TOKEN"})

    c1 = GITHUB_TOKEN_KEY
    checks.append({"name": "github_token_key_literal", "passed": c1 == "GITHUB_TOKEN"})

    c2 = COURIER_URL_KEY
    checks.append(
        {"name": "courier_url_key_literal", "passed": c2 == "AXIOM_COURIER_URL"}
    )

    c3 = COURIER_TOKEN_KEY
    checks.append(
        {"name": "courier_token_key_literal", "passed": c3 == "AXIOM_COURIER_TOKEN"}
    )

    c4 = resolve_github_token(
        lambda k: "tok_gh" if k == "GH_TOKEN" else "tok_github",
        forbidden_helper,
    )
    checks.append(
        {
            "name": "gh_token_precedes_github_token_and_gh_not_called",
            "passed": c4 == "tok_gh",
        }
    )

    c5 = resolve_github_token(
        lambda k: "tok_github" if k == "GITHUB_TOKEN" else None,
        forbidden_helper,
    )
    checks.append(
        {
            "name": "github_token_used_when_gh_token_absent",
            "passed": c5 == "tok_github",
        }
    )

    c6 = resolve_github_token(lambda k: None, lambda: "helper_tok")
    checks.append(
        {"name": "gh_helper_consulted_last", "passed": c6 == "helper_tok"}
    )

    c7 = resolve_github_token(
        lambda k: "  padded_tok  " if k == "GH_TOKEN" else None,
        forbidden_helper,
    )
    checks.append({"name": "resolved_token_trimmed", "passed": c7 == "padded_tok"})

    c8 = resolve_courier_url(
        lambda k: " http://url " if k == "AXIOM_COURIER_URL" else None
    )
    checks.append(
        {"name": "resolve_courier_url_trimmed", "passed": c8 == "http://url"}
    )

    c9 = resolve_courier_token(
        lambda k: " secret_tok " if k == "AXIOM_COURIER_TOKEN" else None
    )
    checks.append(
        {"name": "resolve_courier_token_trimmed", "passed": c9 == "secret_tok"}
    )

    return {
        "case_id": "github-credential-resolution-order-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
