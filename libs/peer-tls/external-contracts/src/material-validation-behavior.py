from __future__ import annotations

from datetime import datetime, timezone

from peer_tls.domain.identity import (
    DnsName,
    ExpectationKind,
    IdentityExpectation,
    SpiffeId,
    TrustDomain,
)
from peer_tls.domain.material import (
    LeafAttributes,
    MaterialTriple,
    PrivateKeyAttributes,
    SubjectAltNames,
    TrustAnchor,
    TrustBundle,
)
from peer_tls.domain.validation import decide_material
from peer_tls.domain.verdict import Rejection, RejectionReason, ValidatedMaterial, ValidityWindow

MINIMUM_CHECKS = 15

MATERIAL_VALIDATION_BEHAVIOR_MATRIX = (
    ("malformed_expectation", "malformed_expectation"),
    ("key_does_not_match_leaf_and_expired", "key_does_not_match_leaf"),
    ("issuer_not_in_trust_bundle", "issuer_not_in_trust_bundle"),
    ("identity_in_wrong_extension", "identity_in_wrong_extension"),
    ("trust_domain_mismatch", "trust_domain_mismatch"),
    ("identity_mismatch", "identity_mismatch"),
    ("not_yet_valid", "not_yet_valid"),
    ("expired", "expired"),
    ("accept_valid_material", "validated"),
    ("validity_window_contains_true", True),
    ("validity_window_contains_false", False),
    ("validity_window_seconds_to_expiry_negative", True),
    ("accepted_window_not_before_is_the_leaf_not_before", "2026-01-10T00:00:00+00:00"),
    ("accepted_window_not_after_is_the_leaf_not_after", "2026-02-10T00:00:00+00:00"),
    ("accepted_window_contains_the_validating_instant", True),
)


def verify_material_validation_behavior() -> dict:
    checks = []

    valid_from = datetime(2026, 1, 10, 0, 0, 0, tzinfo=timezone.utc)
    valid_to = datetime(2026, 2, 10, 0, 0, 0, tzinfo=timezone.utc)
    instant_now = datetime(2026, 1, 15, 12, 0, 0, tzinfo=timezone.utc)

    good_leaf = LeafAttributes(
        subject_alt_names=SubjectAltNames(
            dns_names=(DnsName("service.example.com"),),
            uris=("spiffe://example.com/workload",),
        ),
        not_before=valid_from,
        not_after=valid_to,
        public_key_fingerprint="fp123",
        issuer_key_id="issuer1",
        common_name=None,
    )
    good_key = PrivateKeyAttributes(public_key_fingerprint="fp123")
    good_trust = TrustBundle(anchors=(TrustAnchor(key_id="issuer1", label="ca1"),))
    good_triple = MaterialTriple(leaf=good_leaf, key=good_key, trust=good_trust)

    peer_exp = IdentityExpectation(
        kind=ExpectationKind.PEER,
        spiffe_id=SpiffeId(TrustDomain("example.com"), "workload"),
    )
    serving_exp = IdentityExpectation(
        kind=ExpectationKind.SERVING,
        dns_names=(DnsName("service.example.com"),),
    )

    # 1. malformed_expectation
    malformed_exp = IdentityExpectation(kind=ExpectationKind.SERVING, dns_names=())
    v1 = decide_material(good_triple, malformed_exp, instant_now)
    obs1 = v1.reason.value if isinstance(v1, Rejection) else "validated"
    exp1 = MATERIAL_VALIDATION_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": MATERIAL_VALIDATION_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. key_does_not_match_leaf_and_expired (trips Rule 2 KEY_DOES_NOT_MATCH_LEAF and Rule 8 EXPIRED; Rule 2 wins)
    bad_key = PrivateKeyAttributes(public_key_fingerprint="fp999")
    mismatch_key_triple = MaterialTriple(leaf=good_leaf, key=bad_key, trust=good_trust)
    instant_expired = datetime(2026, 3, 1, 0, 0, 0, tzinfo=timezone.utc)
    v2 = decide_material(mismatch_key_triple, peer_exp, instant_expired)
    obs2 = v2.reason.value if isinstance(v2, Rejection) else "validated"
    exp2 = MATERIAL_VALIDATION_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": MATERIAL_VALIDATION_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. issuer_not_in_trust_bundle
    bad_trust = TrustBundle(anchors=(TrustAnchor(key_id="issuer99", label="ca99"),))
    bad_trust_triple = MaterialTriple(leaf=good_leaf, key=good_key, trust=bad_trust)
    v3 = decide_material(bad_trust_triple, peer_exp, instant_now)
    obs3 = v3.reason.value if isinstance(v3, Rejection) else "validated"
    exp3 = MATERIAL_VALIDATION_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": MATERIAL_VALIDATION_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. identity_in_wrong_extension
    cn_only_leaf = LeafAttributes(
        subject_alt_names=SubjectAltNames(dns_names=(), uris=()),
        not_before=valid_from,
        not_after=valid_to,
        public_key_fingerprint="fp123",
        issuer_key_id="issuer1",
        common_name="service.example.com",
    )
    cn_triple = MaterialTriple(leaf=cn_only_leaf, key=good_key, trust=good_trust)
    v4 = decide_material(cn_triple, serving_exp, instant_now)
    obs4 = v4.reason.value if isinstance(v4, Rejection) else "validated"
    exp4 = MATERIAL_VALIDATION_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": MATERIAL_VALIDATION_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. trust_domain_mismatch
    other_td_exp = IdentityExpectation(
        kind=ExpectationKind.PEER,
        spiffe_id=SpiffeId(TrustDomain("other.com"), "workload"),
    )
    v5 = decide_material(good_triple, other_td_exp, instant_now)
    obs5 = v5.reason.value if isinstance(v5, Rejection) else "validated"
    exp5 = MATERIAL_VALIDATION_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": MATERIAL_VALIDATION_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. identity_mismatch
    other_dns_exp = IdentityExpectation(
        kind=ExpectationKind.SERVING,
        dns_names=(DnsName("other.example.com"),),
    )
    v6 = decide_material(good_triple, other_dns_exp, instant_now)
    obs6 = v6.reason.value if isinstance(v6, Rejection) else "validated"
    exp6 = MATERIAL_VALIDATION_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": MATERIAL_VALIDATION_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. not_yet_valid
    instant_early = datetime(2026, 1, 1, 0, 0, 0, tzinfo=timezone.utc)
    v7 = decide_material(good_triple, peer_exp, instant_early)
    obs7 = v7.reason.value if isinstance(v7, Rejection) else "validated"
    exp7 = MATERIAL_VALIDATION_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": MATERIAL_VALIDATION_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. expired
    v8 = decide_material(good_triple, peer_exp, instant_expired)
    obs8 = v8.reason.value if isinstance(v8, Rejection) else "validated"
    exp8 = MATERIAL_VALIDATION_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": MATERIAL_VALIDATION_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. accept_valid_material
    v9 = decide_material(good_triple, peer_exp, instant_now)
    obs9 = "validated" if isinstance(v9, ValidatedMaterial) else (v9.reason.value if isinstance(v9, Rejection) else "unknown")
    exp9 = MATERIAL_VALIDATION_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": MATERIAL_VALIDATION_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. validity_window_contains_true
    win = ValidityWindow(not_before=valid_from, not_after=valid_to)
    obs10 = win.contains(instant_now)
    exp10 = MATERIAL_VALIDATION_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": MATERIAL_VALIDATION_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. validity_window_contains_false
    obs11 = win.contains(instant_expired)
    exp11 = MATERIAL_VALIDATION_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": MATERIAL_VALIDATION_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. validity_window_seconds_to_expiry_negative
    instant_after_expiry = datetime(2026, 2, 11, 0, 0, 0, tzinfo=timezone.utc)
    secs = win.seconds_to_expiry(instant_after_expiry)
    obs12 = secs < 0
    exp12 = MATERIAL_VALIDATION_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": MATERIAL_VALIDATION_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. accepted_window_not_before_is_the_leaf_not_before
    obs13 = v9.window.not_before.isoformat() if isinstance(v9, ValidatedMaterial) else ""
    exp13 = MATERIAL_VALIDATION_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": MATERIAL_VALIDATION_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. accepted_window_not_after_is_the_leaf_not_after
    obs14 = v9.window.not_after.isoformat() if isinstance(v9, ValidatedMaterial) else ""
    exp14 = MATERIAL_VALIDATION_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": MATERIAL_VALIDATION_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. accepted_window_contains_the_validating_instant
    obs15 = v9.window.contains(instant_now) if isinstance(v9, ValidatedMaterial) else False
    exp15 = MATERIAL_VALIDATION_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": MATERIAL_VALIDATION_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    return {
        "case_id": "material-validation-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
