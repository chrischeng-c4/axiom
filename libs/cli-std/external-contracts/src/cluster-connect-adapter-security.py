from __future__ import annotations

from cli_std.application.token_resolution import resolve_token, uses_cluster
from cli_std.domain.authz import Role, TokenClaims, select_token
from cli_std.domain.registry import bearer_secrets, role_from_name
from cli_std.infrastructure.secret_data import (
    TOKEN_REGISTRY_SECRET_KEY,
    MissingDataKey,
    NotBase64,
    cr_tokens_secret,
    secret_data_bytes,
)

MINIMUM_CHECKS = 11

CLUSTER_CONNECT_ADAPTER_SECURITY_MATRIX = [
    ("secret_data_bytes_strict_base64_refuses_invalid_padding", ("NotBase64", "token-registry.json")),
    ("secret_data_bytes_missing_key_returns_error", ("MissingDataKey", "token-registry.json")),
    ("identities_section_not_parsed_as_bearer_tokens", {}),
    ("select_token_rejects_insufficient_role", None),
    ("select_token_rejects_missing_grant", None),
    ("role_from_name_is_case_sensitive", (None, None)),
    ("explicit_token_short_circuits_without_calling_secret_reader", "exp"),
    ("uses_cluster_returns_false_when_explicit_token_present", False),
    ("uses_cluster_returns_true_when_explicit_token_absent_and_params_present", True),
    ("cr_tokens_secret_extracts_spec_field", "sec1"),
    ("cr_tokens_secret_returns_none_when_field_absent", None),
]


def verify_cluster_connect_adapter_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    res0 = secret_data_bytes({"data": {"token-registry.json": "Zg===="}}, TOKEN_REGISTRY_SECRET_KEY)
    c0 = ("NotBase64", res0.key) if isinstance(res0, NotBase64) else res0
    checks.append({"name": "secret_data_bytes_strict_base64_refuses_invalid_padding", "passed": c0 == ("NotBase64", "token-registry.json")})

    res1 = secret_data_bytes({"data": {}}, TOKEN_REGISTRY_SECRET_KEY)
    c1 = ("MissingDataKey", res1.key) if isinstance(res1, MissingDataKey) else res1
    checks.append({"name": "secret_data_bytes_missing_key_returns_error", "passed": c1 == ("MissingDataKey", "token-registry.json")})

    doc_id = {"identities": {"id1": {"subject": "u1", "roles": {"*": "admin"}}}}
    c2 = bearer_secrets(doc_id)
    checks.append({"name": "identities_section_not_parsed_as_bearer_tokens", "passed": c2 == {}})

    c3 = select_token({"t1": TokenClaims("u1", {"res": Role.READ, "*": Role.WRITE})}, Role.WRITE, "res")
    checks.append({"name": "select_token_rejects_insufficient_role", "passed": c3 is None})

    c4 = select_token({"t1": TokenClaims("u1", {"other": Role.ADMIN})}, Role.ADMIN, "res")
    checks.append({"name": "select_token_rejects_missing_grant", "passed": c4 is None})

    r1 = role_from_name("READ")
    r2 = role_from_name("Admin")
    c5 = (r1.value if isinstance(r1, Role) else None, r2.value if isinstance(r2, Role) else None)
    checks.append({"name": "role_from_name_is_case_sensitive", "passed": c5 == (None, None)})

    def raising_reader(ns: str, n: str) -> object:
        raise RuntimeError("should not be called")

    c6 = resolve_token("exp", "ns", "sec", Role.READ, None, raising_reader, lambda b: None)
    checks.append({"name": "explicit_token_short_circuits_without_calling_secret_reader", "passed": c6 == "exp"})

    c7 = uses_cluster("exp", "ns", "sec")
    checks.append({"name": "uses_cluster_returns_false_when_explicit_token_present", "passed": c7 == False})

    c8 = uses_cluster(None, "ns", "sec")
    checks.append({"name": "uses_cluster_returns_true_when_explicit_token_absent_and_params_present", "passed": c8 == True})

    c9 = cr_tokens_secret({"spec": {"tokensSecret": "sec1"}})
    checks.append({"name": "cr_tokens_secret_extracts_spec_field", "passed": c9 == "sec1"})

    c10 = cr_tokens_secret({"spec": {}})
    checks.append({"name": "cr_tokens_secret_returns_none_when_field_absent", "passed": c10 is None})

    return {
        "case_id": "cluster-connect-adapter-security",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
