from __future__ import annotations

from service_auth.application.delegate_authorization import (
    AuthRejection,
    DelegatedCache,
    authorize_delegated,
    judge,
    make_config,
)
from service_auth.domain.cache_policy import CachePolicy, classify
from service_auth.domain.review import AccessReviewOutcome, ResourceAttributes, TokenReviewOutcome
from service_auth.domain.service_account import (
    PrincipalRejection,
    ReviewedIdentity,
    ServiceAccountRef,
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

MINIMUM_CHECKS = 15

DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX = (
    ("an_unauthenticated_review_reports_the_authentication_rule", "not_authenticated"),
    ("a_wrong_audience_alone_reports_the_audience_rule", "audience_mismatch"),
    ("a_bad_identity_shape_alone_reports_the_shape_rule", "not_a_service_account"),
    ("audience_and_shape_both_wrong_reports_the_audience_rule", "audience_mismatch"),
    ("unauthenticated_outranks_every_later_rule", "not_authenticated"),
    (
        "a_well_formed_credential_is_admitted_as_its_service_account",
        ("lumen-system", "lumen-api"),
    ),
    (
        "access_review_truth_table_over_allowed_and_denied",
        (True, False, False, False),
    ),
    ("an_allow_inside_its_lifetime_classifies_as_a_hit", "hit"),
    ("an_allow_past_its_lifetime_but_inside_the_window_is_stale", "stale"),
    ("an_allow_past_lifetime_plus_window_is_a_miss", "miss"),
    ("a_denial_uses_the_shorter_lifetime_at_the_same_age", ("hit", "stale")),
    (
        "the_ordinary_lookup_never_returns_an_entry_past_its_lifetime",
        "absent",
    ),
    ("the_stale_lookup_returns_the_entry_inside_the_window", True),
    (
        "the_revocation_bound_is_the_allow_lifetime_plus_the_stale_window",
        360,
    ),
    (
        "a_cached_allow_answers_without_reaching_the_backend",
        ("authenticated", 0),
    ),
)


def verify_delegated_kubernetes_authorization_behavior() -> dict:
    checks = []

    # 1. an_unauthenticated_review_reports_the_authentication_rule
    exp1 = DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[0][1]
    j1 = judge(
        make_config([AUDIENCE], POLICY),
        TokenReviewOutcome(
            authenticated=False,
            identity=ReviewedIdentity(
                username=GOOD_USERNAME, uid="u1", groups=(), extra=()
            ),
            audiences=(AUDIENCE,),
        ),
    )
    obs1 = j1.value if isinstance(j1, (AuthRejection, PrincipalRejection)) else "admitted"
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. a_wrong_audience_alone_reports_the_audience_rule
    exp2 = DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[1][1]
    j2 = judge(
        make_config([AUDIENCE], POLICY),
        TokenReviewOutcome(
            authenticated=True,
            identity=ReviewedIdentity(
                username=GOOD_USERNAME, uid="u1", groups=(), extra=()
            ),
            audiences=("some-other-audience",),
        ),
    )
    obs2 = j2.value if isinstance(j2, (AuthRejection, PrincipalRejection)) else "admitted"
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. a_bad_identity_shape_alone_reports_the_shape_rule
    exp3 = DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[2][1]
    j3 = judge(
        make_config([AUDIENCE], POLICY),
        TokenReviewOutcome(
            authenticated=True,
            identity=ReviewedIdentity(
                username="alice@example.com", uid="u1", groups=(), extra=()
            ),
            audiences=(AUDIENCE,),
        ),
    )
    obs3 = j3.value if isinstance(j3, (AuthRejection, PrincipalRejection)) else "admitted"
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. audience_and_shape_both_wrong_reports_the_audience_rule
    exp4 = DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[3][1]
    j4 = judge(
        make_config([AUDIENCE], POLICY),
        TokenReviewOutcome(
            authenticated=True,
            identity=ReviewedIdentity(
                username="alice@example.com", uid="u1", groups=(), extra=()
            ),
            audiences=("some-other-audience",),
        ),
    )
    obs4 = j4.value if isinstance(j4, (AuthRejection, PrincipalRejection)) else "admitted"
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. unauthenticated_outranks_every_later_rule
    exp5 = DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[4][1]
    j5 = judge(
        make_config([AUDIENCE], POLICY),
        TokenReviewOutcome(
            authenticated=False,
            identity=ReviewedIdentity(
                username="alice@example.com", uid="u1", groups=(), extra=()
            ),
            audiences=("some-other-audience",),
        ),
    )
    obs5 = j5.value if isinstance(j5, (AuthRejection, PrincipalRejection)) else "admitted"
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. a_well_formed_credential_is_admitted_as_its_service_account
    exp6 = DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[5][1]
    j6 = judge(
        make_config([AUDIENCE], POLICY),
        TokenReviewOutcome(
            authenticated=True,
            identity=ReviewedIdentity(
                username=GOOD_USERNAME, uid="u1", groups=(), extra=()
            ),
            audiences=(AUDIENCE,),
        ),
    )
    obs6 = (j6.namespace, j6.name) if isinstance(j6, ServiceAccountRef) else j6.value
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. access_review_truth_table_over_allowed_and_denied
    exp7 = DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[6][1]
    obs7 = tuple(
        AccessReviewOutcome(allowed=a, denied=d).is_allowed()
        for (a, d) in ((True, False), (True, True), (False, False), (False, True))
    )
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. an_allow_inside_its_lifetime_classifies_as_a_hit
    exp8 = DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[7][1]
    obs8 = classify(POLICY, stored_at=1000, now=1299, allowed=True).value
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. an_allow_past_its_lifetime_but_inside_the_window_is_stale
    exp9 = DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[8][1]
    obs9 = classify(POLICY, stored_at=1000, now=1301, allowed=True).value
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. an_allow_past_lifetime_plus_window_is_a_miss
    exp10 = DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[9][1]
    obs10 = classify(POLICY, stored_at=1000, now=1361, allowed=True).value
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. a_denial_uses_the_shorter_lifetime_at_the_same_age
    exp11 = DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[10][1]
    obs11 = (
        classify(POLICY, stored_at=1000, now=1045, allowed=True).value,
        classify(POLICY, stored_at=1000, now=1045, allowed=False).value,
    )
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. the_ordinary_lookup_never_returns_an_entry_past_its_lifetime
    exp12 = DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[11][1]
    cache12 = DelegatedCache(policy=POLICY, entries={})
    cache12.put("token12", ATTRS, True, 1000)
    res12 = cache12.get("token12", ATTRS, 1301)
    obs12 = res12 if res12 is not None else "absent"
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. the_stale_lookup_returns_the_entry_inside_the_window
    exp13 = DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[12][1]
    cache13 = DelegatedCache(policy=POLICY, entries={})
    cache13.put("token13", ATTRS, True, 1000)
    res13 = cache13.get_stale("token13", ATTRS, 1301)
    obs13 = res13 if res13 is not None else False
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. the_revocation_bound_is_the_allow_lifetime_plus_the_stale_window
    exp14 = DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[13][1]
    obs14 = POLICY.revocation_bound_seconds()
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    # 15. a_cached_allow_answers_without_reaching_the_backend
    exp15 = DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[14][1]
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
    cache15.put("token15", ATTRS, True, 1000)
    out15 = authorize_delegated(cfg15, bk15, clk15, cache15, "token15", ATTRS)
    obs15 = (out15.value, bk15.token_calls)
    checks.append(
        {
            "name": DELEGATED_KUBERNETES_AUTHORIZATION_BEHAVIOR_MATRIX[14][0],
            "expected": exp15,
            "observed": obs15,
            "passed": obs15 == exp15,
        }
    )

    return {
        "case_id": "delegated-kubernetes-authorization-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
