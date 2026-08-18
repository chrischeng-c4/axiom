from __future__ import annotations

from service_k8s.domain.profile import (
    CertificateIdentity,
    CertificateProfile,
    ProfileError,
)
from service_k8s.domain.purpose import Purpose
from service_k8s.domain.scope import InstanceScope

MINIMUM_CHECKS = 12

SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX = (
    ("a_foreign_namespace_dns_name_is_refused", "ForeignDnsName"),
    ("a_namespace_prefix_is_not_a_namespace_match", "ForeignDnsName"),
    ("a_public_dns_name_is_refused", "PublicDnsName"),
    ("an_empty_name_set_is_refused", "NoNames"),
    ("a_peer_profile_without_a_spiffe_uri_is_refused", "PeerNeedsSpiffeUri"),
    ("a_foreign_trust_domain_spiffe_uri_is_refused", "ForeignSpiffeUri"),
    ("a_foreign_namespace_spiffe_uri_is_refused", "ForeignSpiffeUri"),
    (
        "every_refusal_names_the_offending_value",
        (
            "lumen.example.com",
            "lumen.lumen-prod.svc.cluster.local",
            "spiffe://axiom.internal/ns/other/sa/lumen",
            "spiffe://axiom.internal/ns/lumen/",
        ),
    ),
    ("one_bad_name_among_good_ones_still_refuses", "ForeignDnsName"),
    (
        "a_lifetime_outside_the_bounds_is_refused_at_both_ends",
        ("LifetimeOutOfBounds", "LifetimeOutOfBounds"),
    ),
    (
        "a_renewal_window_with_no_room_to_retry_is_refused",
        ("RenewWindowTooNarrow", "RenewWindowTooWide"),
    ),
    ("jitter_wider_than_the_window_is_refused", "JitterExceedsWindow"),
)

SCOPE = InstanceScope(
    namespace="lumen", instance="lumen-0", trust_domain="axiom.internal"
)
NAMES = ("lumen-0.lumen.svc.cluster.local", "lumen.lumen.svc")
URI = "spiffe://axiom.internal/ns/lumen/sa/lumen"


def _profile(
    purpose: Purpose = Purpose.SERVING,
    names: tuple[str, ...] = NAMES,
    uri: str | None = URI,
    common_name: str = "lumen-0",
    lifetime_secs: int = 3600,
    renew_before_secs: int = 600,
    renew_jitter_secs: int = 0,
) -> CertificateProfile:
    return CertificateProfile(
        scope=SCOPE,
        purpose=purpose,
        common_name=common_name,
        identity=CertificateIdentity(dns_names=names, spiffe_uri=uri),
        lifetime_secs=lifetime_secs,
        renew_before_secs=renew_before_secs,
        renew_jitter_secs=renew_jitter_secs,
    )


def _refusal(**kwargs) -> str:
    try:
        _profile(**kwargs)
    except ProfileError as exc:
        return type(exc).__name__
    return "built"


def verify_scope_validated_certificate_identity_security() -> dict:
    checks = []

    # 1. a_foreign_namespace_dns_name_is_refused
    exp1 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[0][1]
    obs1 = _refusal(names=("lumen-0.other.svc.cluster.local",))
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. a_namespace_prefix_is_not_a_namespace_match
    exp2 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[1][1]
    obs2 = _refusal(names=("lumen.lumen-prod.svc.cluster.local",))
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. a_public_dns_name_is_refused
    exp3 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[2][1]
    obs3 = _refusal(names=("lumen.example.com",))
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. an_empty_name_set_is_refused
    exp4 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[3][1]
    obs4 = _refusal(names=())
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. a_peer_profile_without_a_spiffe_uri_is_refused
    exp5 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[4][1]
    obs5 = _refusal(purpose=Purpose.PEER, uri=None)
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. a_foreign_trust_domain_spiffe_uri_is_refused
    exp6 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[5][1]
    obs6 = _refusal(uri="spiffe://other.internal/ns/lumen/sa/lumen")
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. a_foreign_namespace_spiffe_uri_is_refused
    exp7 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[6][1]
    obs7 = _refusal(uri="spiffe://axiom.internal/ns/other/sa/lumen")
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. every_refusal_names_the_offending_value
    exp8 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[7][1]
    offending8 = []
    try:
        _profile(names=("lumen.example.com",))
    except ProfileError as exc:
        offending8.append(exc.name)
    try:
        _profile(names=("lumen.lumen-prod.svc.cluster.local",))
    except ProfileError as exc:
        offending8.append(exc.name)
    try:
        _profile(uri="spiffe://axiom.internal/ns/other/sa/lumen")
    except ProfileError as exc:
        offending8.append(exc.uri)
        offending8.append(exc.expected_prefix)
    obs8 = tuple(offending8)
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. one_bad_name_among_good_ones_still_refuses
    exp9 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[8][1]
    obs9 = _refusal(
        names=(
            "lumen-0.lumen.svc.cluster.local",
            "lumen.lumen-prod.svc.cluster.local",
            "lumen.lumen.svc",
        )
    )
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. a_lifetime_outside_the_bounds_is_refused_at_both_ends
    exp10 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[9][1]
    obs10 = (_refusal(lifetime_secs=299), _refusal(lifetime_secs=604801))
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. a_renewal_window_with_no_room_to_retry_is_refused
    exp11 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[10][1]
    obs11 = (_refusal(renew_before_secs=599), _refusal(renew_before_secs=3600))
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. jitter_wider_than_the_window_is_refused
    exp12 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[11][1]
    obs12 = _refusal(renew_before_secs=600, renew_jitter_secs=601)
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_SECURITY_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    return {
        "case_id": "scope-validated-certificate-identity-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
