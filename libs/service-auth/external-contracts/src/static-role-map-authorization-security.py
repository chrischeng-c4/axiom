from __future__ import annotations

from service_auth.application.authorize_request import AuthorizeRequest, principal_for_bearer
from service_auth.domain.claims import TokenClaims, resolve_role
from service_auth.domain.principal import DenialReason, TokenPrincipal, ensure
from service_auth.domain.registry import (
    Registry,
    lookup_identity,
    lookup_secret,
    reserved_subject_violation,
)
from service_auth.domain.role import Role

RESERVED_SUBJECTS = ("system:admin", "system:anonymous", "system:masters", "root")

SHARED_KEY = "shared@example.com"
collision = Registry(
    tokens={
        SHARED_KEY: TokenClaims(subject="token-subject", roles={"docs": Role.READ}),
        "secret-only": TokenClaims(subject="s2", roles={"docs": Role.READ}),
    },
    identities={
        SHARED_KEY: TokenClaims(
            subject="identity-subject", roles={"docs": Role.ADMIN}
        ),
        "identity-only@example.com": TokenClaims(
            subject="i2", roles={"docs": Role.ADMIN}
        ),
    },
)

MINIMUM_CHECKS = 15

STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX = (
    ("secret_lookup_returns_only_the_token_namespace_grant", "token-subject"),
    ("identity_lookup_returns_only_the_identity_namespace_grant", "identity-subject"),
    ("a_string_granted_only_as_an_identity_is_not_a_secret", "unresolved"),
    ("a_string_granted_only_as_a_secret_is_not_an_identity", "unresolved"),
    ("the_two_namespaces_disagree_by_construction", ("read", "admin")),
    (
        "reserved_subject_in_tokens_is_named_by_section_key_and_subject",
        ("tokens", "admin-key", "system:admin"),
    ),
    (
        "reserved_subject_in_identities_is_named_by_its_own_section",
        ("identities", "ops@example.com", "root"),
    ),
    ("a_violation_in_both_namespaces_is_reported_against_tokens", "tokens"),
    ("every_reserved_name_the_contract_lists_is_refused", 4),
    ("a_registry_free_of_reserved_subjects_reports_no_violation", "no_violation"),
    ("an_ungranted_resource_is_denied_for_a_known_principal", "insufficient_role"),
    ("a_required_but_absent_credential_is_a_refusal", "missing_bearer"),
    ("an_absent_credential_is_open_only_when_auth_is_not_required", "OpenPrincipal"),
    (
        "no_combination_of_required_and_presented_credential_allows_an_unknown_one",
        ("missing_bearer", "unknown_bearer", "OpenPrincipal", "unknown_bearer"),
    ),
    ("an_unrecognized_credential_is_classified_never_echoed", True),
)


def verify_static_role_map_authorization_security() -> dict:
    checks = []

    # 1. secret_lookup_returns_only_the_token_namespace_grant
    exp1 = STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[0][1]
    c1 = lookup_secret(collision, SHARED_KEY)
    obs1 = c1.subject if c1 is not None else "unresolved"
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. identity_lookup_returns_only_the_identity_namespace_grant
    exp2 = STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[1][1]
    c2 = lookup_identity(collision, SHARED_KEY)
    obs2 = c2.subject if c2 is not None else "unresolved"
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. a_string_granted_only_as_an_identity_is_not_a_secret
    exp3 = STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[2][1]
    c3 = lookup_secret(collision, "identity-only@example.com")
    obs3 = c3.subject if c3 is not None else "unresolved"
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. a_string_granted_only_as_a_secret_is_not_an_identity
    exp4 = STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[3][1]
    c4 = lookup_identity(collision, "secret-only")
    obs4 = c4.subject if c4 is not None else "unresolved"
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. the_two_namespaces_disagree_by_construction
    exp5 = STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[4][1]
    s5 = lookup_secret(collision, SHARED_KEY)
    i5 = lookup_identity(collision, SHARED_KEY)
    tr5 = resolve_role(s5, "docs") if s5 is not None else None
    ir5 = resolve_role(i5, "docs") if i5 is not None else None
    obs5 = (
        tr5.value if tr5 is not None else "unresolved",
        ir5.value if ir5 is not None else "unresolved",
    )
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. reserved_subject_in_tokens_is_named_by_section_key_and_subject
    exp6 = STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[5][1]
    reg6 = Registry(
        tokens={"admin-key": TokenClaims(subject="system:admin", roles={})},
        identities={},
    )
    v6 = reserved_subject_violation(reg6, RESERVED_SUBJECTS)
    obs6 = v6 if v6 is not None else "no_violation"
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. reserved_subject_in_identities_is_named_by_its_own_section
    exp7 = STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[6][1]
    reg7 = Registry(
        tokens={},
        identities={"ops@example.com": TokenClaims(subject="root", roles={})},
    )
    v7 = reserved_subject_violation(reg7, RESERVED_SUBJECTS)
    obs7 = v7 if v7 is not None else "no_violation"
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. a_violation_in_both_namespaces_is_reported_against_tokens
    exp8 = STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[7][1]
    reg8 = Registry(
        tokens={"t_key": TokenClaims(subject="system:admin", roles={})},
        identities={"i_key": TokenClaims(subject="root", roles={})},
    )
    v8 = reserved_subject_violation(reg8, RESERVED_SUBJECTS)
    obs8 = v8[0] if v8 is not None else "no_violation"
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. every_reserved_name_the_contract_lists_is_refused
    exp9 = STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[8][1]
    obs9 = sum(
        1
        for name in RESERVED_SUBJECTS
        if reserved_subject_violation(
            Registry(
                tokens={"k": TokenClaims(subject=name, roles={})}, identities={}
            ),
            RESERVED_SUBJECTS,
        )
        is not None
    )
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. a_registry_free_of_reserved_subjects_reports_no_violation
    exp10 = STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[9][1]
    reg10 = Registry(
        tokens={"user": TokenClaims(subject="alice", roles={})}, identities={}
    )
    v10 = reserved_subject_violation(reg10, RESERVED_SUBJECTS)
    obs10 = v10 if v10 is not None else "no_violation"
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. an_ungranted_resource_is_denied_for_a_known_principal
    exp11 = STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[10][1]
    p11 = TokenPrincipal(TokenClaims(subject="alice", roles={"docs": Role.READ}))
    d11 = ensure(p11, "secrets", Role.READ)
    obs11 = d11.reason.value if d11 is not None else "allowed"
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. a_required_but_absent_credential_is_a_refusal
    exp12 = STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[11][1]
    svc12 = AuthorizeRequest(
        registry=Registry(tokens={}, identities={}), auth_required=True
    )
    res12 = principal_for_bearer(svc12, None)
    obs12 = res12.value if isinstance(res12, DenialReason) else type(res12).__name__
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. an_absent_credential_is_open_only_when_auth_is_not_required
    exp13 = STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[12][1]
    svc13 = AuthorizeRequest(
        registry=Registry(tokens={}, identities={}), auth_required=False
    )
    res13 = principal_for_bearer(svc13, None)
    obs13 = (
        type(res13).__name__
        if not isinstance(res13, DenialReason)
        else res13.value
    )
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. no_combination_of_required_and_presented_credential_allows_an_unknown_one
    exp14 = STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[13][1]
    reg14 = Registry(tokens={}, identities={})
    combos14 = [
        (True, None),
        (True, "not-in-the-registry"),
        (False, None),
        (False, "not-in-the-registry"),
    ]
    obs14 = tuple(
        r.value if isinstance(r, DenialReason) else type(r).__name__
        for req, sec in combos14
        for r in [
            principal_for_bearer(
                AuthorizeRequest(registry=reg14, auth_required=req), sec
            )
        ]
    )
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    # 15. an_unrecognized_credential_is_classified_never_echoed
    exp15 = STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[14][1]
    sec15 = "sk-live-DO-NOT-ECHO"
    res15 = principal_for_bearer(
        AuthorizeRequest(registry=Registry(tokens={}, identities={}), auth_required=True),
        sec15,
    )
    obs15 = isinstance(res15, DenialReason) and sec15 not in repr(res15)
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_SECURITY_MATRIX[14][0],
            "expected": exp15,
            "observed": obs15,
            "passed": obs15 == exp15,
        }
    )

    return {
        "case_id": "static-role-map-authorization-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
