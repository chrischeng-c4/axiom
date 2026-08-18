from __future__ import annotations

from datetime import datetime, timedelta

from service_k8s.application.rotation import (
    Action,
    Desired,
    IssuerId,
    Observed,
    ObservedLeaf,
    next_action,
    renew_at,
    retry_after,
)
from service_k8s.domain.profile import CertificateIdentity, CertificateProfile
from service_k8s.domain.purpose import Purpose
from service_k8s.domain.scope import InstanceScope

MINIMUM_CHECKS = 13

MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX = (
    ("trust_is_published_before_anything_is_issued", "PublishTrustBundle"),
    (
        "no_reachable_action_removes_the_current_leaf",
        (
            "PublishTrustBundle",
            "Issue",
            "Issue",
            "AwaitActivation",
            "RetireIssuers",
            "Wait",
        ),
    ),
    (
        "the_action_type_has_no_removal_variant",
        ("AwaitActivation", "Issue", "PublishTrustBundle", "RetireIssuers", "Wait"),
    ),
    (
        "retirement_is_gated_on_observed_activation_not_on_a_write",
        ("AwaitActivation", "RetireIssuers"),
    ),
    ("a_mismatched_activation_fingerprint_does_not_retire", "AwaitActivation"),
    (
        "an_issuer_rotation_publishes_the_new_anchor_before_issuing",
        ("PublishTrustBundle", "Issue"),
    ),
    ("a_long_expired_leaf_is_reissued_never_removed", ("Issue", "Expired")),
    ("the_decision_does_not_depend_on_call_history", ("Issue", "Issue", "Issue")),
    ("the_decision_moves_only_because_the_instant_moved", ("Wait", "Issue")),
    (
        "renewal_jitter_never_pushes_renewal_past_expiry",
        (True, True, True, True, True),
    ),
    ("backoff_never_exceeds_the_five_minute_ceiling", (True, 300)),
    ("backoff_never_drops_below_the_base", 5),
    (
        "an_untrusted_issuer_is_published_before_even_an_expired_leaf_is_replaced",
        "PublishTrustBundle",
    ),
)

SCOPE = InstanceScope(
    namespace="lumen", instance="lumen-0", trust_domain="axiom.internal"
)
NAMES = ("lumen-0.lumen.svc.cluster.local",)
URI = "spiffe://axiom.internal/ns/lumen/sa/lumen"

PROFILE = CertificateProfile(
    scope=SCOPE,
    purpose=Purpose.SERVING,
    common_name="lumen-0",
    identity=CertificateIdentity(dns_names=NAMES, spiffe_uri=URI),
    lifetime_secs=3600,
    renew_before_secs=600,
    renew_jitter_secs=0,
)

JITTERED = CertificateProfile(
    scope=SCOPE,
    purpose=Purpose.SERVING,
    common_name="lumen-0",
    identity=CertificateIdentity(dns_names=NAMES, spiffe_uri=URI),
    lifetime_secs=3600,
    renew_before_secs=600,
    renew_jitter_secs=300,
)

ISSUER_A = IssuerId("issuer-a")
ISSUER_B = IssuerId("issuer-b")
T0 = datetime(2026, 1, 1, 0, 0, 0)
EXPIRY = datetime(2026, 1, 1, 1, 0, 0)
DESIRED = Desired(PROFILE, ISSUER_A)


def _leaf(
    issuer: IssuerId = ISSUER_A,
    fingerprint: str = "fp-new",
    identity_digest: str | None = None,
    not_after: datetime = EXPIRY,
) -> ObservedLeaf:
    return ObservedLeaf(
        issuer=issuer,
        not_before=not_after - timedelta(hours=1),
        not_after=not_after,
        fingerprint=fingerprint,
        identity_digest=(
            PROFILE.identity_digest() if identity_digest is None else identity_digest
        ),
    )


def _name(action) -> str:
    return type(action).__name__


def verify_memoryless_rotation_decision_security() -> dict:
    checks = []

    # 1. trust_is_published_before_anything_is_issued
    exp1 = MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[0][1]
    obs1 = _name(next_action(DESIRED, Observed(), T0))
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. no_reachable_action_removes_the_current_leaf
    exp2 = MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[1][1]
    observations2 = (
        Observed(),
        Observed(trust_bundle=(ISSUER_A,)),
        Observed(leaf=_leaf(issuer=ISSUER_B), trust_bundle=(ISSUER_A, ISSUER_B)),
        Observed(leaf=_leaf(), trust_bundle=(ISSUER_A, ISSUER_B)),
        Observed(
            leaf=_leaf(),
            trust_bundle=(ISSUER_A, ISSUER_B),
            activated_fingerprint="fp-new",
        ),
        Observed(leaf=_leaf(), trust_bundle=(ISSUER_A,)),
    )
    names2 = []
    for observed in observations2:
        names2.append(_name(next_action(DESIRED, observed, T0)))
    obs2 = tuple(names2)
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. the_action_type_has_no_removal_variant
    exp3 = MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[2][1]
    obs3 = tuple(sorted(variant.__name__ for variant in Action.__args__))
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. retirement_is_gated_on_observed_activation_not_on_a_write
    exp4 = MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[3][1]
    obs4 = (
        _name(
            next_action(
                DESIRED,
                Observed(leaf=_leaf(), trust_bundle=(ISSUER_A, ISSUER_B)),
                T0,
            )
        ),
        _name(
            next_action(
                DESIRED,
                Observed(
                    leaf=_leaf(),
                    trust_bundle=(ISSUER_A, ISSUER_B),
                    activated_fingerprint="fp-new",
                ),
                T0,
            )
        ),
    )
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. a_mismatched_activation_fingerprint_does_not_retire
    exp5 = MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[4][1]
    obs5 = _name(
        next_action(
            DESIRED,
            Observed(
                leaf=_leaf(),
                trust_bundle=(ISSUER_A, ISSUER_B),
                activated_fingerprint="fp-old",
            ),
            T0,
        )
    )
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. an_issuer_rotation_publishes_the_new_anchor_before_issuing
    exp6 = MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[5][1]
    desired6 = Desired(PROFILE, ISSUER_B)
    obs6 = (
        _name(
            next_action(
                desired6, Observed(leaf=_leaf(), trust_bundle=(ISSUER_A,)), T0
            )
        ),
        _name(
            next_action(
                desired6,
                Observed(leaf=_leaf(), trust_bundle=(ISSUER_A, ISSUER_B)),
                T0,
            )
        ),
    )
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. a_long_expired_leaf_is_reissued_never_removed
    exp7 = MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[6][1]
    action7 = next_action(
        DESIRED,
        Observed(leaf=_leaf(), trust_bundle=(ISSUER_A,)),
        EXPIRY + timedelta(days=30),
    )
    obs7 = (_name(action7), action7.reason.token)
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. the_decision_does_not_depend_on_call_history
    exp8 = MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[7][1]
    names8 = []
    for _ in (1, 2, 3):
        names8.append(
            _name(next_action(DESIRED, Observed(trust_bundle=(ISSUER_A,)), T0))
        )
    obs8 = tuple(names8)
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. the_decision_moves_only_because_the_instant_moved
    exp9 = MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[8][1]
    observed9 = Observed(leaf=_leaf(), trust_bundle=(ISSUER_A,))
    obs9 = (
        _name(next_action(DESIRED, observed9, T0)),
        _name(next_action(DESIRED, observed9, T0 + timedelta(minutes=55))),
    )
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. renewal_jitter_never_pushes_renewal_past_expiry
    exp10 = MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[9][1]
    inside10 = []
    for fingerprint in ("fp-one", "fp-two", "fp-three", "fp-four", "fp-five"):
        inside10.append(
            renew_at(JITTERED, _leaf(fingerprint=fingerprint)) < EXPIRY
        )
    obs10 = tuple(inside10)
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. backoff_never_exceeds_the_five_minute_ceiling
    exp11 = MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[10][1]
    seconds11 = []
    for failures in range(0, 21):
        seconds11.append(int(retry_after(failures).total_seconds()))
    obs11 = (max(seconds11) <= 300, max(seconds11))
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. backoff_never_drops_below_the_base
    exp12 = MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[11][1]
    seconds12 = []
    for failures in range(0, 21):
        seconds12.append(int(retry_after(failures).total_seconds()))
    obs12 = min(seconds12)
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. an_untrusted_issuer_is_published_before_even_an_expired_leaf_is_replaced
    exp13 = MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[12][1]
    obs13 = _name(
        next_action(
            DESIRED,
            Observed(
                leaf=_leaf(
                    identity_digest="stale-digest",
                    not_after=T0 - timedelta(days=1),
                ),
                trust_bundle=(),
            ),
            T0,
        )
    )
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_SECURITY_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    return {
        "case_id": "memoryless-rotation-decision-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
