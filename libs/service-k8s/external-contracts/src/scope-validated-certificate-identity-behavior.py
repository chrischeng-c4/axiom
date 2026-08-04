from __future__ import annotations

from service_k8s.domain.profile import CertificateIdentity, CertificateProfile
from service_k8s.domain.purpose import Purpose
from service_k8s.domain.scope import InstanceScope

MINIMUM_CHECKS = 14

SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX = (
    ("a_serving_leaf_carries_server_auth_alone", ("serverAuth",)),
    ("a_peer_leaf_carries_both_directions", ("serverAuth", "clientAuth")),
    ("the_usage_set_is_exact_not_a_superset", (1, 2)),
    (
        "the_secret_name_is_derived_from_the_scope_and_the_purpose",
        ("lumen-0-serving-tls", "lumen-0-peer-tls"),
    ),
    (
        "the_spiffe_prefix_is_derived_from_trust_domain_and_namespace",
        "spiffe://axiom.internal/ns/lumen/",
    ),
    ("a_scope_covers_only_itself", (True, False, False)),
    ("the_identity_digest_ignores_dns_name_order", True),
    ("the_identity_digest_changes_when_a_name_changes", False),
    ("the_identity_digest_ignores_lifetime_and_renewal_cadence", True),
    ("the_identity_digest_separates_the_two_purposes", False),
    ("the_identity_digest_is_a_sixty_four_character_hex_string", (64, True)),
    ("both_cluster_internal_suffix_forms_are_accepted", 2),
    ("the_maximum_lifetime_is_accepted", 604800),
    ("jitter_equal_to_the_whole_window_is_accepted", (600, 600)),
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


def verify_scope_validated_certificate_identity_behavior() -> dict:
    checks = []

    # 1. a_serving_leaf_carries_server_auth_alone
    exp1 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[0][1]
    obs1 = tuple(u.token for u in _profile().extended_key_usages())
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. a_peer_leaf_carries_both_directions
    exp2 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[1][1]
    obs2 = tuple(u.token for u in _profile(purpose=Purpose.PEER).extended_key_usages())
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. the_usage_set_is_exact_not_a_superset
    exp3 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[2][1]
    obs3 = (
        len(_profile().extended_key_usages()),
        len(_profile(purpose=Purpose.PEER).extended_key_usages()),
    )
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. the_secret_name_is_derived_from_the_scope_and_the_purpose
    exp4 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[3][1]
    obs4 = (
        _profile().secret_name(),
        _profile(purpose=Purpose.PEER).secret_name(),
    )
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. the_spiffe_prefix_is_derived_from_trust_domain_and_namespace
    exp5 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[4][1]
    obs5 = SCOPE.spiffe_prefix()
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. a_scope_covers_only_itself
    exp6 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[5][1]
    obs6 = (
        SCOPE.covers(
            InstanceScope(
                namespace="lumen", instance="lumen-0", trust_domain="axiom.internal"
            )
        ),
        SCOPE.covers(
            InstanceScope(
                namespace="other", instance="lumen-0", trust_domain="axiom.internal"
            )
        ),
        SCOPE.covers(
            InstanceScope(
                namespace="lumen", instance="lumen-1", trust_domain="axiom.internal"
            )
        ),
    )
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. the_identity_digest_ignores_dns_name_order
    exp7 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[6][1]
    obs7 = (
        _profile(
            names=("lumen-0.lumen.svc.cluster.local", "lumen.lumen.svc")
        ).identity_digest()
        == _profile(
            names=("lumen.lumen.svc", "lumen-0.lumen.svc.cluster.local")
        ).identity_digest()
    )
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. the_identity_digest_changes_when_a_name_changes
    exp8 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[7][1]
    obs8 = (
        _profile().identity_digest()
        == _profile(
            names=("lumen-1.lumen.svc.cluster.local", "lumen.lumen.svc")
        ).identity_digest()
    )
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. the_identity_digest_ignores_lifetime_and_renewal_cadence
    exp9 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[8][1]
    obs9 = (
        _profile().identity_digest()
        == _profile(
            lifetime_secs=7200, renew_before_secs=1200, renew_jitter_secs=300
        ).identity_digest()
    )
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. the_identity_digest_separates_the_two_purposes
    exp10 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[9][1]
    obs10 = (
        _profile().identity_digest()
        == _profile(purpose=Purpose.PEER).identity_digest()
    )
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. the_identity_digest_is_a_sixty_four_character_hex_string
    exp11 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[10][1]
    digest11 = _profile().identity_digest()
    obs11 = (len(digest11), set(digest11) <= set("0123456789abcdef"))
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. both_cluster_internal_suffix_forms_are_accepted
    exp12 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[11][1]
    obs12 = len(
        _profile(
            names=("lumen-0.lumen.svc.cluster.local", "lumen.lumen.svc")
        ).identity.dns_names
    )
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. the_maximum_lifetime_is_accepted
    exp13 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[12][1]
    obs13 = _profile(lifetime_secs=604800).lifetime_secs
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. jitter_equal_to_the_whole_window_is_accepted
    exp14 = SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[13][1]
    profile14 = _profile(renew_before_secs=600, renew_jitter_secs=600)
    obs14 = (profile14.renew_before_secs, profile14.renew_jitter_secs)
    checks.append(
        {
            "name": SCOPE_VALIDATED_CERTIFICATE_IDENTITY_BEHAVIOR_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    return {
        "case_id": "scope-validated-certificate-identity-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
