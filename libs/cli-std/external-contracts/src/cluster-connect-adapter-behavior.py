from __future__ import annotations

from cli_std.application.token_resolution import resolve_token
from cli_std.domain.authz import Role, TokenClaims, select_token
from cli_std.domain.registry import bearer_secrets, is_namespaced, role_from_name

MINIMUM_CHECKS = 11

CLUSTER_CONNECT_ADAPTER_BEHAVIOR_MATRIX = [
    ("role_covers_hierarchy_admin_covers_read", True),
    ("role_covers_hierarchy_read_does_not_cover_admin", False),
    ("role_covers_hierarchy_write_does_not_cover_admin", False),
    ("discriminator_flat_registry_with_tokens_key", False),
    ("bearer_secrets_parses_flat_registry_tokens_key", {"tokens": {"subject": "u1", "roles": {"*": 3}}}),
    ("bearer_secrets_parses_namespaced_registry", {"t1": {"subject": "u1", "roles": {"*": 3}}}),
    ("select_token_prefers_resource_grant_over_wildcard", "t1"),
    ("select_token_consults_wildcard_when_no_resource_grant", "t_wild"),
    ("resolve_token_prefers_explicit_token", "exp_tok"),
    ("resolve_token_returns_none_when_namespace_missing", None),
    ("role_from_name_parsing", (1, 2, 3)),
]


def verify_cluster_connect_adapter_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    c0 = Role.ADMIN.covers(Role.READ)
    checks.append({"name": "role_covers_hierarchy_admin_covers_read", "passed": c0 == True})

    c1 = Role.READ.covers(Role.ADMIN)
    checks.append({"name": "role_covers_hierarchy_read_does_not_cover_admin", "passed": c1 == False})

    c2 = Role.WRITE.covers(Role.ADMIN)
    checks.append({"name": "role_covers_hierarchy_write_does_not_cover_admin", "passed": c2 == False})

    c3 = is_namespaced({"tokens": {"subject": "u1", "roles": {"*": "admin"}}})
    checks.append({"name": "discriminator_flat_registry_with_tokens_key", "passed": c3 == False})

    res4 = bearer_secrets({"tokens": {"subject": "u1", "roles": {"*": "admin"}}})
    c4 = (
        {
            k: {
                "subject": v.subject,
                "roles": {rk: rv.value for rk, rv in v.roles.items()},
            }
            for k, v in res4.items()
        }
        if isinstance(res4, dict)
        else None
    )
    checks.append({"name": "bearer_secrets_parses_flat_registry_tokens_key", "passed": c4 == {"tokens": {"subject": "u1", "roles": {"*": 3}}}})

    res5 = bearer_secrets({"tokens": {"t1": {"subject": "u1", "roles": {"*": "admin"}}}})
    c5 = (
        {
            k: {
                "subject": v.subject,
                "roles": {rk: rv.value for rk, rv in v.roles.items()},
            }
            for k, v in res5.items()
        }
        if isinstance(res5, dict)
        else None
    )
    checks.append({"name": "bearer_secrets_parses_namespaced_registry", "passed": c5 == {"t1": {"subject": "u1", "roles": {"*": 3}}}})

    reg6 = {"t1": TokenClaims("u1", {"res1": Role.WRITE, "*": Role.READ})}
    c6 = select_token(reg6, Role.WRITE, "res1")
    checks.append({"name": "select_token_prefers_resource_grant_over_wildcard", "passed": c6 == "t1"})

    reg7 = {"t_wild": TokenClaims("u1", {"*": Role.WRITE})}
    c7 = select_token(reg7, Role.WRITE, "res2")
    checks.append({"name": "select_token_consults_wildcard_when_no_resource_grant", "passed": c7 == "t_wild"})

    c8 = resolve_token("exp_tok", "ns", "sec", Role.READ, None, lambda ns, n: None, lambda b: None)
    checks.append({"name": "resolve_token_prefers_explicit_token", "passed": c8 == "exp_tok"})

    c9 = resolve_token(None, None, "sec", Role.READ, None, lambda ns, n: None, lambda b: None)
    checks.append({"name": "resolve_token_returns_none_when_namespace_missing", "passed": c9 is None})

    r_read = role_from_name("read")
    r_write = role_from_name("write")
    r_admin = role_from_name("admin")
    c10 = (
        r_read.value if isinstance(r_read, Role) else None,
        r_write.value if isinstance(r_write, Role) else None,
        r_admin.value if isinstance(r_admin, Role) else None,
    )
    checks.append({"name": "role_from_name_parsing", "passed": c10 == (1, 2, 3)})

    return {
        "case_id": "cluster-connect-adapter-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
        "checks": checks,
    }
