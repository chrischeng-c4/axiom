from __future__ import annotations

from datetime import datetime, timezone
import inspect
import typing

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
from peer_tls.domain.verdict import (
    MaterialVerdict,
    Rejection,
    RejectionReason,
    ValidatedMaterial,
)

MINIMUM_CHECKS = 11

MATERIAL_VALIDATION_SECURITY_MATRIX = (
    ("near_miss_wrong_dns_name", "identity_mismatch"),
    ("near_miss_wrong_spiffe_trust_domain", "trust_domain_mismatch"),
    ("near_miss_identity_in_wrong_extension", "identity_in_wrong_extension"),
    ("near_miss_expired_leaf", "expired"),
    ("near_miss_key_mismatch", "key_does_not_match_leaf"),
    ("verdict_domain_admits_exactly_two_shapes", True),
    ("decide_material_parameter_count_is_three", True),
    ("decide_material_has_no_permissive_or_override_params", True),
    ("every_refusal_carries_a_nonempty_detail", True),
    ("refusal_details_are_distinct_per_reason", True),
    ("all_eight_refusal_reasons_are_observed", 8),
)


def verify_material_validation_security() -> dict:
    checks = []

    valid_from = datetime(2026, 1, 10, 0, 0, 0, tzinfo=timezone.utc)
    valid_to = datetime(2026, 2, 10, 0, 0, 0, tzinfo=timezone.utc)
    instant_now = datetime(2026, 1, 15, 12, 0, 0, tzinfo=timezone.utc)

    good_key = PrivateKeyAttributes(public_key_fingerprint="fp123")
    good_trust = TrustBundle(anchors=(TrustAnchor(key_id="issuer1", label="ca1"),))

    # 1. near_miss_wrong_dns_name
    leaf_dns = LeafAttributes(
        subject_alt_names=SubjectAltNames(dns_names=(DnsName("actual.example.com"),)),
        not_before=valid_from,
        not_after=valid_to,
        public_key_fingerprint="fp123",
        issuer_key_id="issuer1",
    )
    triple_dns = MaterialTriple(leaf=leaf_dns, key=good_key, trust=good_trust)
    exp_dns = IdentityExpectation(kind=ExpectationKind.SERVING, dns_names=(DnsName("expected.example.com"),))
    v1 = decide_material(triple_dns, exp_dns, instant_now)
    obs1 = v1.reason.value if isinstance(v1, Rejection) else "validated"
    exp1 = MATERIAL_VALIDATION_SECURITY_MATRIX[0][1]
    checks.append({"name": MATERIAL_VALIDATION_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. near_miss_wrong_spiffe_trust_domain
    leaf_spiffe = LeafAttributes(
        subject_alt_names=SubjectAltNames(uris=("spiffe://actual-td.com/app",)),
        not_before=valid_from,
        not_after=valid_to,
        public_key_fingerprint="fp123",
        issuer_key_id="issuer1",
    )
    triple_spiffe = MaterialTriple(leaf=leaf_spiffe, key=good_key, trust=good_trust)
    exp_spiffe = IdentityExpectation(kind=ExpectationKind.PEER, spiffe_id=SpiffeId(TrustDomain("expected-td.com"), "app"))
    v2 = decide_material(triple_spiffe, exp_spiffe, instant_now)
    obs2 = v2.reason.value if isinstance(v2, Rejection) else "validated"
    exp2 = MATERIAL_VALIDATION_SECURITY_MATRIX[1][1]
    checks.append({"name": MATERIAL_VALIDATION_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. near_miss_identity_in_wrong_extension
    leaf_cn = LeafAttributes(
        subject_alt_names=SubjectAltNames(dns_names=(), uris=()),
        not_before=valid_from,
        not_after=valid_to,
        public_key_fingerprint="fp123",
        issuer_key_id="issuer1",
        common_name="service.example.com",
    )
    triple_cn = MaterialTriple(leaf=leaf_cn, key=good_key, trust=good_trust)
    exp_cn = IdentityExpectation(kind=ExpectationKind.SERVING, dns_names=(DnsName("service.example.com"),))
    v3 = decide_material(triple_cn, exp_cn, instant_now)
    obs3 = v3.reason.value if isinstance(v3, Rejection) else "validated"
    exp3 = MATERIAL_VALIDATION_SECURITY_MATRIX[2][1]
    checks.append({"name": MATERIAL_VALIDATION_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. near_miss_expired_leaf (build dedicated leaf whose identity MATCHES exp_dns, then expire it)
    leaf_expired = LeafAttributes(
        subject_alt_names=SubjectAltNames(dns_names=(DnsName("expected.example.com"),)),
        not_before=valid_from,
        not_after=valid_to,
        public_key_fingerprint="fp123",
        issuer_key_id="issuer1",
    )
    triple_expired = MaterialTriple(leaf=leaf_expired, key=good_key, trust=good_trust)
    instant_expired = datetime(2026, 3, 1, 0, 0, 0, tzinfo=timezone.utc)
    v4 = decide_material(triple_expired, exp_dns, instant_expired)
    obs4 = v4.reason.value if isinstance(v4, Rejection) else "validated"
    exp4 = MATERIAL_VALIDATION_SECURITY_MATRIX[3][1]
    checks.append({"name": MATERIAL_VALIDATION_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. near_miss_key_mismatch
    bad_key = PrivateKeyAttributes(public_key_fingerprint="fp999")
    triple_key_mismatch = MaterialTriple(leaf=leaf_dns, key=bad_key, trust=good_trust)
    v5 = decide_material(triple_key_mismatch, exp_dns, instant_now)
    obs5 = v5.reason.value if isinstance(v5, Rejection) else "validated"
    exp5 = MATERIAL_VALIDATION_SECURITY_MATRIX[4][1]
    checks.append({"name": MATERIAL_VALIDATION_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. verdict_domain_admits_exactly_two_shapes
    union_args = set(typing.get_args(MaterialVerdict))
    obs6 = union_args == {ValidatedMaterial, Rejection}
    exp6 = MATERIAL_VALIDATION_SECURITY_MATRIX[5][1]
    checks.append({"name": MATERIAL_VALIDATION_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. decide_material_parameter_count_is_three
    sig = inspect.signature(decide_material)
    obs7 = len(sig.parameters) == 3
    exp7 = MATERIAL_VALIDATION_SECURITY_MATRIX[6][1]
    checks.append({"name": MATERIAL_VALIDATION_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. decide_material_has_no_permissive_or_override_params
    forbidden_terms = {"override", "force", "permissive", "strict"}
    param_names = set(sig.parameters.keys())
    has_forbidden = any(
        term in p.lower() for p in param_names for term in forbidden_terms
    ) or any(p.lower().startswith("allow_") for p in param_names)
    obs8 = not has_forbidden
    exp8 = MATERIAL_VALIDATION_SECURITY_MATRIX[7][1]
    checks.append({"name": MATERIAL_VALIDATION_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # Additional 3 refusal fixtures to complete all 8 RejectionReasons
    # Refusal 6: malformed expectation -> MALFORMED_EXPECTATION
    v_malformed = decide_material(triple_dns, IdentityExpectation(kind=ExpectationKind.SERVING, dns_names=()), instant_now)

    # Refusal 7: unknown issuer -> ISSUER_NOT_IN_TRUST_BUNDLE
    leaf_untrusted = LeafAttributes(
        subject_alt_names=SubjectAltNames(dns_names=(DnsName("expected.example.com"),)),
        not_before=valid_from,
        not_after=valid_to,
        public_key_fingerprint="fp123",
        issuer_key_id="unknown_issuer",
    )
    v_untrusted = decide_material(MaterialTriple(leaf=leaf_untrusted, key=good_key, trust=good_trust), exp_dns, instant_now)

    # Refusal 8: early instant -> NOT_YET_VALID
    instant_early = datetime(2026, 1, 1, 0, 0, 0, tzinfo=timezone.utc)
    v_not_yet = decide_material(triple_expired, exp_dns, instant_early)

    near_miss_verdicts = [v1, v2, v3, v4, v5]
    all_refusals = near_miss_verdicts + [v_malformed, v_untrusted, v_not_yet]

    # 9. every_refusal_carries_a_nonempty_detail
    obs9 = all(isinstance(v, Rejection) and isinstance(v.detail, str) and len(v.detail) > 0 for v in all_refusals)
    exp9 = MATERIAL_VALIDATION_SECURITY_MATRIX[8][1]
    checks.append({"name": MATERIAL_VALIDATION_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. refusal_details_are_distinct_per_reason
    obs10 = len({v.detail for v in all_refusals if isinstance(v, Rejection)}) == len(all_refusals)
    exp10 = MATERIAL_VALIDATION_SECURITY_MATRIX[9][1]
    checks.append({"name": MATERIAL_VALIDATION_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. all_eight_refusal_reasons_are_observed
    obs11 = len({v.reason.value for v in all_refusals if isinstance(v, Rejection)})
    exp11 = MATERIAL_VALIDATION_SECURITY_MATRIX[10][1]
    checks.append({"name": MATERIAL_VALIDATION_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    return {
        "case_id": "material-validation-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
