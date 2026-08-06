from __future__ import annotations

import dataclasses

from service_auth.application.authorize_request import (
    AuthorizeRequest,
    authorize,
    principal_for_bearer,
)
from service_auth.application.reload_registry import (
    ReloadableRegistry,
    reload_documents,
    validate,
)
from service_auth.domain.audit import (
    AUTHORIZATION_EVENT_FIELDS,
    REGISTRY_RELOAD_EVENT_FIELDS,
    AuthorizationEvent,
    RegistryReloadEvent,
)
from service_auth.domain.claims import TokenClaims
from service_auth.domain.registry import (
    Registry,
    reserved_subject_violation,
)
from service_auth.domain.role import Role
from service_auth.infrastructure.ports import AuthEventSink, RegistrySource


class TextSource:
    def __init__(self, name: str, text: str) -> None:
        self.name = name
        self._text = text

    def read(self) -> str:
        return self._text


class ListSink:
    def __init__(self) -> None:
        self.events: list[object] = []

    def record(self, event: object) -> None:
        self.events.append(event)


SECRET = "sk-live-DO-NOT-ECHO-0123456789"
RESERVED_SUBJECTS = ("system:masters", "system:anonymous")


def token_registry(
    key: str, subject: str, resource: str = "docs", role: Role = Role.READ
) -> Registry:
    return Registry(
        tokens={key: TokenClaims(subject=subject, roles={resource: role})},
        identities={},
    )


def identity_registry(
    key: str, subject: str, resource: str = "docs", role: Role = Role.READ
) -> Registry:
    return Registry(
        tokens={},
        identities={key: TokenClaims(subject=subject, roles={resource: role})},
    )


MINIMUM_CHECKS = 16

CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX = (
    (
        "each_validation_violation_reports_its_own_reason",
        (
            "required_but_empty",
            "empty_key",
            "empty_subject",
            "empty_resource",
            "identity_key_not_an_email",
            "ok",
        ),
    ),
    (
        "an_empty_registry_is_admitted_when_authentication_is_not_required",
        "ok",
    ),
    (
        "a_blank_key_and_a_blank_subject_are_not_the_same_violation",
        ("empty_key", "empty_subject"),
    ),
    (
        "an_identity_key_that_is_not_an_email_is_rejected_although_nothing_else_is_wrong",
        "identity_key_not_an_email",
    ),
    ("the_identity_email_rule_admits_a_well_formed_address", "ok"),
    (
        "a_reserved_subject_in_the_token_namespace_is_reported_with_its_key_and_subject",
        ("tokens", "k", "system:masters"),
    ),
    (
        "a_reserved_subject_in_the_identity_namespace_is_reported_with_its_key_and_subject",
        ("identities", "a@b.com", "system:masters"),
    ),
    ("a_registry_free_of_reserved_subjects_reports_no_violation", "none"),
    ("a_reserved_subject_is_refused_before_anything_is_published", (0, 0)),
    (
        "the_authorization_event_carries_exactly_its_five_declared_fields",
        ("outcome", "reason", "subject", "resource", "needed"),
    ),
    (
        "the_reload_event_carries_exactly_its_four_declared_fields",
        ("applied", "revision", "entries", "failure"),
    ),
    (
        "the_declared_field_tuples_agree_with_the_dataclasses_themselves",
        (True, True),
    ),
    (
        "a_denied_authorization_records_no_trace_of_the_credential",
        (False, False),
    ),
    ("an_allowed_authorization_records_no_trace_of_the_credential", False),
    (
        "a_reload_event_records_no_trace_of_the_registry_it_published",
        (False, True),
    ),
    (
        "a_parse_failure_records_no_trace_of_the_document_that_failed",
        False,
    ),
)


def verify_credential_reload_audit_security() -> dict:
    checks = []

    # 1. each_validation_violation_reports_its_own_reason
    exp1 = CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[0][1]
    r1_1 = Registry(tokens={}, identities={})
    r1_2 = token_registry("   ", "alice")
    r1_3 = token_registry("k", "   ")
    r1_4 = token_registry("k", "alice", resource="   ")
    r1_5 = identity_registry("not-an-email", "bob")
    r1_6 = identity_registry("a@b.com", "bob")
    corpus1 = [
        (True, r1_1),
        (False, r1_2),
        (False, r1_3),
        (False, r1_4),
        (False, r1_5),
        (False, r1_6),
    ]
    obs1 = tuple(
        v if (v := validate(req, reg)) is not None else "ok"
        for req, reg in corpus1
    )
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. an_empty_registry_is_admitted_when_authentication_is_not_required
    exp2 = CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[1][1]
    v2 = validate(False, Registry(tokens={}, identities={}))
    obs2 = v2 if v2 is not None else "ok"
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. a_blank_key_and_a_blank_subject_are_not_the_same_violation
    exp3 = CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[2][1]
    v3_1 = validate(False, token_registry("  ", "alice"))
    v3_2 = validate(False, token_registry("k", "  "))
    obs3 = (
        v3_1 if v3_1 is not None else "ok",
        v3_2 if v3_2 is not None else "ok",
    )
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. an_identity_key_that_is_not_an_email_is_rejected_although_nothing_else_is_wrong
    exp4 = CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[3][1]
    v4 = validate(False, identity_registry("service-account", "svc"))
    obs4 = v4 if v4 is not None else "ok"
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. the_identity_email_rule_admits_a_well_formed_address
    exp5 = CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[4][1]
    v5 = validate(False, identity_registry("svc@example.com", "svc"))
    obs5 = v5 if v5 is not None else "ok"
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. a_reserved_subject_in_the_token_namespace_names_that_namespace
    exp6 = CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[5][1]
    v6 = reserved_subject_violation(
        token_registry("k", "system:masters"), RESERVED_SUBJECTS
    )
    obs6 = v6 if v6 is not None else "none"
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. a_reserved_subject_in_the_identity_namespace_names_that_namespace
    exp7 = CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[6][1]
    v7 = reserved_subject_violation(
        identity_registry("a@b.com", "system:masters"), RESERVED_SUBJECTS
    )
    obs7 = v7 if v7 is not None else "none"
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. a_registry_free_of_reserved_subjects_reports_no_violation
    exp8 = CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[7][1]
    v8 = reserved_subject_violation(
        token_registry("k", "alice"), RESERVED_SUBJECTS
    )
    obs8 = v8 if v8 is not None else "none"
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. a_reserved_subject_is_refused_before_anything_is_published
    exp9 = CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[8][1]
    st9 = ReloadableRegistry(reserved=RESERVED_SUBJECTS)
    snk9 = ListSink()
    res9_text = (
        '{"tokens": {"k": {"subject": "system:anonymous", "roles": {"docs":'
        ' "read"}}}}'
    )
    reload_documents(st9, [TextSource("x", res9_text)], snk9)
    obs9 = (st9.revision, st9.registry.len())
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. the_authorization_event_carries_exactly_its_five_declared_fields
    exp10 = CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[9][1]
    obs10 = tuple(f.name for f in dataclasses.fields(AuthorizationEvent))
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. the_reload_event_carries_exactly_its_four_declared_fields
    exp11 = CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[10][1]
    obs11 = tuple(f.name for f in dataclasses.fields(RegistryReloadEvent))
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. the_declared_field_tuples_agree_with_the_dataclasses_themselves
    exp12 = CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[11][1]
    obs12 = (
        tuple(f.name for f in dataclasses.fields(AuthorizationEvent))
        == AUTHORIZATION_EVENT_FIELDS,
        tuple(f.name for f in dataclasses.fields(RegistryReloadEvent))
        == REGISTRY_RELOAD_EVENT_FIELDS,
    )
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. a_denied_authorization_records_no_trace_of_the_credential
    exp13 = CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[12][1]
    reg13 = Registry(
        tokens={SECRET: TokenClaims(subject="alice", roles={"docs": Role.READ})},
        identities={},
    )
    svc13 = AuthorizeRequest(registry=reg13, auth_required=True)
    p13 = principal_for_bearer(svc13, SECRET)
    snk13 = ListSink()
    authorize(svc13, p13, "docs", Role.ADMIN, snk13)
    ev13 = snk13.events[0]
    obs13 = (SECRET in repr(ev13), SECRET in str(getattr(ev13, "subject", "")))
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. an_allowed_authorization_records_no_trace_of_the_credential
    exp14 = CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[13][1]
    reg14 = Registry(
        tokens={SECRET: TokenClaims(subject="alice", roles={"docs": Role.READ})},
        identities={},
    )
    svc14 = AuthorizeRequest(registry=reg14, auth_required=True)
    p14 = principal_for_bearer(svc14, SECRET)
    snk14 = ListSink()
    authorize(svc14, p14, "docs", Role.READ, snk14)
    ev14 = snk14.events[0]
    obs14 = SECRET in repr(ev14)
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    # 15. a_reload_event_records_no_trace_of_the_registry_it_published
    exp15 = CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[14][1]
    st15 = ReloadableRegistry()
    snk15 = ListSink()
    doc15 = (
        '{"tokens": {"'
        + SECRET
        + '": {"subject": "alice", "roles": {"docs": "read"}}}}'
    )
    reload_documents(st15, [TextSource("x", doc15)], snk15)
    ev15 = snk15.events[0]
    obs15 = (
        SECRET in repr(ev15),
        getattr(ev15, "entries", 0) == 1,
    )
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[14][0],
            "expected": exp15,
            "observed": obs15,
            "passed": obs15 == exp15,
        }
    )

    # 16. a_parse_failure_records_no_trace_of_the_document_that_failed
    exp16 = CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[15][1]
    st16 = ReloadableRegistry()
    snk16 = ListSink()
    bad16 = (
        '{"tokens": {"'
        + SECRET
        + '": {"subject": "x", "roles": {"docs": "wizard"}}}}'
    )
    reload_documents(st16, [TextSource("x", bad16)], snk16)
    ev16 = snk16.events[0]
    obs16 = SECRET in repr(ev16)
    checks.append(
        {
            "name": CREDENTIAL_RELOAD_AUDIT_SECURITY_MATRIX[15][0],
            "expected": exp16,
            "observed": obs16,
            "passed": obs16 == exp16,
        }
    )

    return {
        "case_id": "credential-reload-audit-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
