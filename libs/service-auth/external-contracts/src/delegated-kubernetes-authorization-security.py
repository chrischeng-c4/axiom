from __future__ import annotations

from service_auth.application.delegate_authorization import (
    AuthRejection,
    DelegatedCache,
    MissingAudienceError,
    authorize_delegated,
    fingerprint,
    make_config,
)
from service_auth.domain.cache_policy import CachePolicy
from service_auth.domain.review import (
    AccessReviewOutcome,
    ResourceAttributes,
    ReviewError,
    TokenReviewOutcome,
)
from service_auth.domain.service_account import (
    UNAUTHENTICATED_GROUP,
    PrincipalRejection,
    ReviewedIdentity,
    ServiceAccountRef,
    is_dns1123_label,
    parse_service_account,
    principal_from_review,
)
from service_auth.infrastructure.ports import (
    Clock,
    DelegatedBackendError,
    ReviewBackend,
)


class FakeClock:
    def __init__(self, start: int) -> None:
        self.value = start

    def now_seconds(self) -> int:
        return self.value

    def advance(self, seconds: int) -> None:
        self.value += seconds


class FakeBackend:
    def __init__(self, review, access, token_error=None, access_error=None) -> None:
        self.review = review
        self.access = access
        self.token_error = token_error
        self.access_error = access_error
        self.token_calls = 0
        self.access_calls = 0

    def review_token(self, token, audiences):
        self.token_calls += 1
        if self.token_error is not None:
            raise DelegatedBackendError(self.token_error)
        return self.review

    def review_access(self, identity, attributes):
        self.access_calls += 1
        if self.access_error is not None:
            raise DelegatedBackendError(self.access_error)
        return self.access


POLICY = CachePolicy(
    allow_ttl_seconds=300,
    deny_ttl_seconds=30,
    stale_window_seconds=60,
    max_entries=8192,
)
AUDIENCE = "lumen.axiom.internal"
GOOD_USERNAME = "system:serviceaccount:lumen-system:lumen-api"
ATTRS = ResourceAttributes(
    group="axiom.io",
    namespace="lumen-system",
    resource="indexes",
    name="primary",
    verb="get",
)

LONG = "a" * 63
TOO_LONG = "a" * 64

ADMISSIBLE = (
    "system:serviceaccount:default:lumen",
    "system:serviceaccount:a:b",
    "system:serviceaccount:lumen-system:lumen-api",
    "system:serviceaccount:a1:2b",
    "system:serviceaccount:" + LONG + ":" + LONG,
)

INADMISSIBLE = (
    "",
    "system:anonymous",
    "alice@example.com",
    "system:serviceaccount:onlyone",
    "system:serviceaccount:a:b:c",
    "system:serviceaccount:NS:lumen",
    "system:serviceaccount:default:Lumen",
    "system:serviceaccount::lumen",
    "system:serviceaccount:-ns:lumen",
    "system:serviceaccount:ns-:lumen",
    "system:serviceaccount:ns:lumen_api",
    "system:serviceaccount:" + TOO_LONG + ":lumen",
)

SECRET = "sk-live-DO-NOT-ECHO-0123456789"

MINIMUM_CHECKS = 17

DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX = (
    (
        "the_admissible_corpus_parses_to_its_namespace_and_name",
        (
            ("default", "lumen"),
            ("a", "b"),
            ("lumen-system", "lumen-api"),
            ("a1", "2b"),
            (
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            ),
        ),
    ),
    (
        "the_inadmissible_corpus_reports_its_own_reason_for_each_row",
        (
            "missing_username",
            "anonymous",
            "not_a_service_account",
            "malformed_service_account",
            "malformed_service_account",
            "malformed_service_account",
            "malformed_service_account",
            "malformed_service_account",
            "malformed_service_account",
            "malformed_service_account",
            "malformed_service_account",
            "malformed_service_account",
        ),
    ),
    (
        "the_dns_label_predicate_agrees_with_the_contracts_own_rule",
        (True, True, True, True, False, False, False, False, False, False),
    ),
    (
        "the_rejection_reason_set_is_exactly_the_five_declared_values",
        (
            "not_authenticated",
            "missing_username",
            "anonymous",
            "not_a_service_account",
            "malformed_service_account",
        ),
    ),
    (
        "an_unauthenticated_review_is_rejected_before_its_username_matters",
        "not_authenticated",
    ),
    ("membership_in_the_unauthenticated_group_is_anonymous", "anonymous"),
    (
        "a_configuration_naming_no_audience_is_refused_at_construction",
        "MissingAudienceError",
    ),
    (
        "a_configuration_naming_only_blank_audiences_is_refused",
        "MissingAudienceError",
    ),
    (
        "a_token_review_outage_with_no_cached_decision_is_unavailable",
        "unavailable",
    ),
    (
        "an_access_review_outage_with_no_cached_decision_is_unavailable",
        "unavailable",
    ),
    (
        "a_malformed_response_with_no_cached_decision_is_unavailable",
        "unavailable",
    ),
    ("the_three_outage_classes_never_produce_an_allow", ("unavailable",)),
    ("an_outage_with_a_stale_allow_serves_the_stale_decision", "authenticated"),
    ("an_outage_with_a_stale_denial_serves_the_denial", "denied"),
    (
        "an_empty_credential_is_refused_without_reaching_the_backend",
        ("unauthenticated", 0),
    ),
    (
        "the_credential_never_appears_in_a_cache_key_or_a_fingerprint",
        (False, 12, False),
    ),
    (
        "the_auth_rejection_reason_set_is_exactly_the_one_declared_value",
        ("audience_mismatch",),
    ),
)


def verify_delegated_kubernetes_authorization_security() -> dict:
    checks = []

    # 1. the_admissible_corpus_parses_to_its_namespace_and_name
    exp1 = DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[0][1]
    obs1 = tuple(
        (res.namespace, res.name) if isinstance(res, ServiceAccountRef) else res.value
        for u in ADMISSIBLE
        for res in [parse_service_account(u)]
    )
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. the_inadmissible_corpus_reports_its_own_reason_for_each_row
    exp2 = DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[1][1]
    obs2 = tuple(
        res.value if isinstance(res, PrincipalRejection) else "admitted"
        for u in INADMISSIBLE
        for res in [parse_service_account(u)]
    )
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. the_dns_label_predicate_agrees_with_the_contracts_own_rule
    exp3 = DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[2][1]
    obs3 = tuple(
        is_dns1123_label(s)
        for s in ("a", "a1", "a-b", LONG, "", TOO_LONG, "-a", "a-", "A", "a_b")
    )
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. the_rejection_reason_set_is_exactly_the_five_declared_values
    exp4 = DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[3][1]
    obs4 = tuple(r.value for r in PrincipalRejection)
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. an_unauthenticated_review_is_rejected_before_its_username_matters
    exp5 = DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[4][1]
    res5 = principal_from_review(
        False,
        ReviewedIdentity(
            username=GOOD_USERNAME, uid="u1", groups=(), extra=()
        ),
    )
    obs5 = res5.value if isinstance(res5, PrincipalRejection) else "admitted"
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. membership_in_the_unauthenticated_group_is_anonymous
    exp6 = DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[5][1]
    res6 = principal_from_review(
        True,
        ReviewedIdentity(
            username=GOOD_USERNAME,
            uid="u1",
            groups=(UNAUTHENTICATED_GROUP,),
            extra=(),
        ),
    )
    obs6 = res6.value if isinstance(res6, PrincipalRejection) else "admitted"
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. a_configuration_naming_no_audience_is_refused_at_construction
    exp7 = DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[6][1]
    try:
        make_config((), POLICY)
        obs7 = "no_refusal"
    except MissingAudienceError:
        obs7 = "MissingAudienceError"
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. a_configuration_naming_only_blank_audiences_is_refused
    exp8 = DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[7][1]
    try:
        make_config(("", "   "), POLICY)
        obs8 = "no_refusal"
    except MissingAudienceError:
        obs8 = "MissingAudienceError"
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. a_token_review_outage_with_no_cached_decision_is_unavailable
    exp9 = DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[8][1]
    cfg9 = make_config([AUDIENCE], POLICY)
    bk9 = FakeBackend(review=None, access=None, token_error=ReviewError.TRANSPORT)
    clk9 = FakeClock(1000)
    cache9 = DelegatedCache(policy=POLICY, entries={})
    obs9 = authorize_delegated(cfg9, bk9, clk9, cache9, "tok9", ATTRS).value
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. an_access_review_outage_with_no_cached_decision_is_unavailable
    exp10 = DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[9][1]
    cfg10 = make_config([AUDIENCE], POLICY)
    bk10 = FakeBackend(
        review=TokenReviewOutcome(
            authenticated=True,
            identity=ReviewedIdentity(
                username=GOOD_USERNAME, uid="u1", groups=(), extra=()
            ),
            audiences=(AUDIENCE,),
        ),
        access=None,
        access_error=ReviewError.TRANSPORT,
    )
    clk10 = FakeClock(1000)
    cache10 = DelegatedCache(policy=POLICY, entries={})
    obs10 = authorize_delegated(cfg10, bk10, clk10, cache10, "tok10", ATTRS).value
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. a_malformed_response_with_no_cached_decision_is_unavailable
    exp11 = DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[10][1]
    cfg11 = make_config([AUDIENCE], POLICY)
    bk11 = FakeBackend(review=None, access=None, token_error=ReviewError.MALFORMED)
    clk11 = FakeClock(1000)
    cache11 = DelegatedCache(policy=POLICY, entries={})
    obs11 = authorize_delegated(cfg11, bk11, clk11, cache11, "tok11", ATTRS).value
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. the_three_outage_classes_never_produce_an_allow
    exp12 = DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[11][1]
    obs12 = tuple(sorted(set([obs9, obs10, obs11])))
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. an_outage_with_a_stale_allow_serves_the_stale_decision
    exp13 = DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[12][1]
    cfg13 = make_config([AUDIENCE], POLICY)
    bk13 = FakeBackend(review=None, access=None, token_error=ReviewError.TRANSPORT)
    clk13 = FakeClock(1301)
    cache13 = DelegatedCache(policy=POLICY, entries={})
    cache13.put("tok13", ATTRS, True, 1000)
    obs13 = authorize_delegated(cfg13, bk13, clk13, cache13, "tok13", ATTRS).value
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. an_outage_with_a_stale_denial_serves_the_denial
    exp14 = DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[13][1]
    cfg14 = make_config([AUDIENCE], POLICY)
    bk14 = FakeBackend(review=None, access=None, token_error=ReviewError.TRANSPORT)
    clk14 = FakeClock(1045)
    cache14 = DelegatedCache(policy=POLICY, entries={})
    cache14.put("tok14", ATTRS, False, 1000)
    obs14 = authorize_delegated(cfg14, bk14, clk14, cache14, "tok14", ATTRS).value
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    # 15. an_empty_credential_is_refused_without_reaching_the_backend
    exp15 = DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[14][1]
    cfg15 = make_config([AUDIENCE], POLICY)
    bk15 = FakeBackend(
        review=TokenReviewOutcome(
            authenticated=True,
            identity=ReviewedIdentity(
                username=GOOD_USERNAME, uid="u1", groups=(), extra=()
            ),
            audiences=(AUDIENCE,),
        ),
        access=AccessReviewOutcome(allowed=True, denied=False),
    )
    clk15 = FakeClock(1000)
    cache15 = DelegatedCache(policy=POLICY, entries={})
    out15 = authorize_delegated(cfg15, bk15, clk15, cache15, "", ATTRS)
    obs15 = (out15.value, bk15.token_calls)
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[14][0],
            "expected": exp15,
            "observed": obs15,
            "passed": obs15 == exp15,
        }
    )

    # 16. the_credential_never_appears_in_a_cache_key_or_a_fingerprint
    exp16 = DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[15][1]
    cache16 = DelegatedCache(policy=POLICY, entries={})
    cache16.put(SECRET, ATTRS, True, 1000)
    fp16 = fingerprint(SECRET)
    obs16 = (
        any(SECRET in key[0] for key in cache16.entries),
        len(fp16),
        fp16 == SECRET,
    )
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[15][0],
            "expected": exp16,
            "observed": obs16,
            "passed": obs16 == exp16,
        }
    )

    # 17. the_auth_rejection_reason_set_is_exactly_the_one_declared_value
    exp17 = DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[16][1]
    obs17 = tuple(m.value for m in AuthRejection)
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_SECURITY_MATRIX[16][0],
            "expected": exp17,
            "observed": obs17,
            "passed": obs17 == exp17,
        }
    )

    return {
        "case_id": "delegated-kubernetes-authorization-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
