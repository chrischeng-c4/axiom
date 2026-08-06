from __future__ import annotations

from service_auth.application.authorize_request import AuthorizeRequest, authorize
from service_auth.domain.audit import AuthEvent
from service_auth.domain.claims import WILDCARD_RESOURCE, TokenClaims, resolve_role
from service_auth.domain.principal import (
    AuthorizationOutcome,
    OpenPrincipal,
    TokenPrincipal,
)
from service_auth.domain.registry import Registry, RegistryError, parse, try_merge
from service_auth.domain.role import ROLE_ORDER, Role, covers


class _RecordingSink:
    def __init__(self) -> None:
        self.records: list[AuthEvent] = []

    def record(self, event: AuthEvent) -> None:
        self.records.append(event)


MINIMUM_CHECKS = 14

STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX = (
    (
        "role_coverage_over_all_nine_pairs",
        (True, False, False, True, True, False, True, True, True),
    ),
    ("declared_role_order_is_read_write_admin", ("read", "write", "admin")),
    ("exact_resource_entry_beats_the_wildcard", "write"),
    ("wildcard_applies_only_when_no_exact_entry", "admin"),
    ("resource_named_by_neither_resolves_to_nothing", "unresolved"),
    ("open_principal_allows_without_consulting_any_grant", "allow"),
    ("namespaced_document_splits_into_two_namespaces", (1, 1)),
    ("flat_document_is_read_entirely_as_tokens", (2, 0)),
    ("flat_document_whose_only_key_is_a_section_name", (1, 0)),
    ("merge_of_disjoint_namespaces_unions_them", (2, 1)),
    ("colliding_token_key_is_refused_with_its_reason", "duplicate_registry_key"),
    (
        "colliding_identity_key_is_refused_with_its_reason",
        "duplicate_registry_key",
    ),
    (
        "insufficient_role_denies_and_names_its_reason",
        ("deny", "insufficient_role"),
    ),
    ("authorization_outcome_admits_strictly_two_values", ("allow", "deny")),
)


def verify_static_role_map_authorization_behavior() -> dict:
    checks = []

    # 1. role_coverage_over_all_nine_pairs
    exp1 = STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[0][1]
    obs1 = tuple(
        covers(h, n)
        for h in (Role.READ, Role.WRITE, Role.ADMIN)
        for n in (Role.READ, Role.WRITE, Role.ADMIN)
    )
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. declared_role_order_is_read_write_admin
    exp2 = STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[1][1]
    obs2 = tuple(r.value for r in ROLE_ORDER)
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. exact_resource_entry_beats_the_wildcard
    exp3 = STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[2][1]
    c3 = TokenClaims(
        subject="s", roles={WILDCARD_RESOURCE: Role.ADMIN, "docs": Role.WRITE}
    )
    r3 = resolve_role(c3, "docs")
    obs3 = r3.value if r3 is not None else "unresolved"
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. wildcard_applies_only_when_no_exact_entry
    exp4 = STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[3][1]
    c4 = TokenClaims(
        subject="s", roles={WILDCARD_RESOURCE: Role.ADMIN, "docs": Role.WRITE}
    )
    r4 = resolve_role(c4, "reports")
    obs4 = r4.value if r4 is not None else "unresolved"
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. resource_named_by_neither_resolves_to_nothing
    exp5 = STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[4][1]
    c5 = TokenClaims(subject="s", roles={"docs": Role.READ})
    r5 = resolve_role(c5, "reports")
    obs5 = r5.value if r5 is not None else "unresolved"
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. open_principal_allows_without_consulting_any_grant
    exp6 = STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[5][1]
    svc6 = AuthorizeRequest(
        registry=Registry(tokens={}, identities={}), auth_required=False
    )
    sink6 = _RecordingSink()
    obs6 = authorize(svc6, OpenPrincipal(), "docs", Role.ADMIN, sink6).value
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. namespaced_document_splits_into_two_namespaces
    exp7 = STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[6][1]
    reg7 = parse(
        {
            "tokens": {"t1": {"subject": "ts", "roles": {"docs": "read"}}},
            "identities": {
                "a@example.com": {"subject": "is", "roles": {"docs": "read"}}
            },
        }
    )
    obs7 = (len(reg7.tokens), len(reg7.identities))
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. flat_document_is_read_entirely_as_tokens
    exp8 = STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[7][1]
    reg8 = parse(
        {
            "t1": {"subject": "s1", "roles": {"docs": "read"}},
            "t2": {"subject": "s2", "roles": {"docs": "write"}},
        }
    )
    obs8 = (len(reg8.tokens), len(reg8.identities))
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. flat_document_whose_only_key_is_a_section_name
    exp9 = STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[8][1]
    try:
        reg9 = parse({"tokens": {"subject": "ts", "roles": {"docs": "read"}}})
        obs9: object = (len(reg9.tokens), len(reg9.identities))
    except RegistryError as err9:
        obs9 = err9.reason
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. merge_of_disjoint_namespaces_unions_them
    exp10 = STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[9][1]
    r10_a = Registry(tokens={"t1": TokenClaims("s1", {})}, identities={})
    r10_b = Registry(
        tokens={"t2": TokenClaims("s2", {})},
        identities={"i1": TokenClaims("s3", {})},
    )
    m10 = try_merge(r10_a, r10_b)
    obs10 = (len(m10.tokens), len(m10.identities))
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. colliding_token_key_is_refused_with_its_reason
    exp11 = STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[10][1]
    r11_a = Registry(tokens={"k": TokenClaims("s1", {})}, identities={})
    r11_b = Registry(tokens={"k": TokenClaims("s2", {})}, identities={})
    try:
        try_merge(r11_a, r11_b)
        obs11 = "no_refusal"
    except RegistryError as err11:
        obs11 = err11.reason
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. colliding_identity_key_is_refused_with_its_reason
    exp12 = STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[11][1]
    r12_a = Registry(tokens={}, identities={"id": TokenClaims("s1", {})})
    r12_b = Registry(tokens={}, identities={"id": TokenClaims("s2", {})})
    try:
        try_merge(r12_a, r12_b)
        obs12 = "no_refusal"
    except RegistryError as err12:
        obs12 = err12.reason
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. insufficient_role_denies_and_names_its_reason
    exp13 = STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[12][1]
    c13 = TokenClaims("s13", {"docs": Role.READ})
    p13 = TokenPrincipal(c13)
    svc13 = AuthorizeRequest(registry=Registry({}, {}), auth_required=True)
    sink13 = _RecordingSink()
    out13 = authorize(svc13, p13, "docs", Role.ADMIN, sink13)
    rec13 = sink13.records[-1]
    obs13 = (
        out13.value,
        rec13.reason.value if rec13.reason is not None else "no_reason",
    )
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. authorization_outcome_admits_exactly_two_values
    exp14 = STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[13][1]
    obs14 = tuple(o.value for o in AuthorizationOutcome)
    checks.append(
        {
            "name": STATIC_ROLE_MAP_AUTHORIZATION_BEHAVIOR_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    return {
        "case_id": "static-role-map-authorization-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
