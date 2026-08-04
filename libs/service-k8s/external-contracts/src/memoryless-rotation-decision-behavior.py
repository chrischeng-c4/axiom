from __future__ import annotations

from datetime import datetime, timedelta

from service_k8s.application.rotation import (
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

MINIMUM_CHECKS = 15

MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX = (
    ("an_empty_observation_publishes_trust_first", ("PublishTrustBundle", ("issuer-a",))),
    ("a_trusted_issuer_with_no_leaf_bootstraps", ("Issue", "Bootstrap")),
    ("a_leaf_from_another_issuer_is_an_issuer_rotation", ("Issue", "IssuerRotation")),
    ("a_leaf_whose_identity_moved_is_reissued", ("Issue", "IdentityChanged")),
    ("an_expired_leaf_is_reissued_as_expired", ("Issue", "Expired")),
    ("a_due_leaf_is_reissued_as_a_renewal", ("Issue", "Renewal")),
    (
        "a_leaf_that_is_not_yet_due_waits_until_its_renewal_instant",
        ("Wait", "2026-01-01T00:50:00"),
    ),
    (
        "the_publish_step_is_a_superset_of_what_is_already_trusted",
        ("issuer-a", "issuer-b"),
    ),
    (
        "an_activated_new_leaf_retires_the_stale_issuer",
        ("RetireIssuers", ("issuer-b",)),
    ),
    ("an_unactivated_new_leaf_awaits_activation", ("AwaitActivation", "fp-new", 15)),
    (
        "the_renewal_instant_does_not_move_when_the_controller_restarts",
        (True, "2026-01-01T00:51:05"),
    ),
    (
        "different_certificates_still_spread_out",
        ("2026-01-01T00:52:22", "2026-01-01T00:54:36"),
    ),
    ("zero_jitter_puts_renewal_exactly_at_the_window", "2026-01-01T00:50:00"),
    ("backoff_starts_at_the_base_and_doubles", (5, 10, 20, 40, 80, 160)),
    ("backoff_saturates_at_the_ceiling", (300, 300, 300)),
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


def verify_memoryless_rotation_decision_behavior() -> dict:
    checks = []

    # 1. an_empty_observation_publishes_trust_first
    exp1 = MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[0][1]
    action1 = next_action(DESIRED, Observed(), T0)
    obs1 = (_name(action1), tuple(i.value for i in action1.issuers))
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. a_trusted_issuer_with_no_leaf_bootstraps
    exp2 = MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[1][1]
    action2 = next_action(DESIRED, Observed(trust_bundle=(ISSUER_A,)), T0)
    obs2 = (_name(action2), action2.reason.token)
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. a_leaf_from_another_issuer_is_an_issuer_rotation
    exp3 = MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[2][1]
    action3 = next_action(
        DESIRED,
        Observed(leaf=_leaf(issuer=ISSUER_B), trust_bundle=(ISSUER_A, ISSUER_B)),
        T0,
    )
    obs3 = (_name(action3), action3.reason.token)
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. a_leaf_whose_identity_moved_is_reissued
    exp4 = MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[3][1]
    action4 = next_action(
        DESIRED,
        Observed(leaf=_leaf(identity_digest="stale-digest"), trust_bundle=(ISSUER_A,)),
        T0,
    )
    obs4 = (_name(action4), action4.reason.token)
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. an_expired_leaf_is_reissued_as_expired
    exp5 = MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[4][1]
    action5 = next_action(
        DESIRED, Observed(leaf=_leaf(), trust_bundle=(ISSUER_A,)), EXPIRY
    )
    obs5 = (_name(action5), action5.reason.token)
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. a_due_leaf_is_reissued_as_a_renewal
    exp6 = MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[5][1]
    action6 = next_action(
        DESIRED,
        Observed(leaf=_leaf(), trust_bundle=(ISSUER_A,)),
        T0 + timedelta(minutes=50),
    )
    obs6 = (_name(action6), action6.reason.token)
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. a_leaf_that_is_not_yet_due_waits_until_its_renewal_instant
    exp7 = MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[6][1]
    action7 = next_action(
        DESIRED, Observed(leaf=_leaf(), trust_bundle=(ISSUER_A,)), T0
    )
    obs7 = (_name(action7), action7.until.isoformat())
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. the_publish_step_is_a_superset_of_what_is_already_trusted
    exp8 = MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[7][1]
    action8 = next_action(DESIRED, Observed(trust_bundle=(ISSUER_B,)), T0)
    obs8 = tuple(i.value for i in action8.issuers)
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. an_activated_new_leaf_retires_the_stale_issuer
    exp9 = MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[8][1]
    action9 = next_action(
        DESIRED,
        Observed(
            leaf=_leaf(),
            trust_bundle=(ISSUER_A, ISSUER_B),
            activated_fingerprint="fp-new",
        ),
        T0,
    )
    obs9 = (_name(action9), tuple(i.value for i in action9.issuers))
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. an_unactivated_new_leaf_awaits_activation
    exp10 = MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[9][1]
    action10 = next_action(
        DESIRED,
        Observed(
            leaf=_leaf(), trust_bundle=(ISSUER_A, ISSUER_B), activated_fingerprint=None
        ),
        T0,
    )
    obs10 = (
        _name(action10),
        action10.fingerprint,
        int(action10.recheck_after.total_seconds()),
    )
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. the_renewal_instant_does_not_move_when_the_controller_restarts
    exp11 = MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[10][1]
    first11 = renew_at(JITTERED, _leaf())
    second11 = renew_at(JITTERED, _leaf())
    obs11 = (first11 == second11, second11.isoformat())
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. different_certificates_still_spread_out
    exp12 = MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[11][1]
    obs12 = (
        renew_at(JITTERED, _leaf(fingerprint="fp-one")).isoformat(),
        renew_at(JITTERED, _leaf(fingerprint="fp-two")).isoformat(),
    )
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. zero_jitter_puts_renewal_exactly_at_the_window
    exp13 = MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[12][1]
    obs13 = renew_at(PROFILE, _leaf()).isoformat()
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. backoff_starts_at_the_base_and_doubles
    exp14 = MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[13][1]
    seconds14 = []
    for failures in (0, 1, 2, 3, 4, 5):
        seconds14.append(int(retry_after(failures).total_seconds()))
    obs14 = tuple(seconds14)
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    # 15. backoff_saturates_at_the_ceiling
    exp15 = MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[14][1]
    seconds15 = []
    for failures in (6, 7, 1000000):
        seconds15.append(int(retry_after(failures).total_seconds()))
    obs15 = tuple(seconds15)
    checks.append(
        {
            "name": MEMORYLESS_ROTATION_DECISION_BEHAVIOR_MATRIX[14][0],
            "expected": exp15,
            "observed": obs15,
            "passed": obs15 == exp15,
        }
    )

    return {
        "case_id": "memoryless-rotation-decision-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
