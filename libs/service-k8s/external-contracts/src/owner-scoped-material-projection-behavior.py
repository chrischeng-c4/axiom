from __future__ import annotations

from datetime import datetime, timezone

from service_k8s.application.rotation import Issue, IssueReason, IssuerId, Wait
from service_k8s.application.status import CertificateFacts
from service_k8s.application.trust_bundle import TrustBundle
from service_k8s.domain.purpose import Purpose
from service_k8s.domain.scope import InstanceScope
from service_k8s.infrastructure.projection import (
    CERT_KEY,
    IDENTITY_DIGEST_ANNOTATION,
    IssuedMaterial,
    LEAF_ISSUER_ANNOTATION,
    Owner,
    PRIVATE_KEY_KEY,
    TRUST_BUNDLE_ANNOTATION,
    TRUST_BUNDLE_KEY,
    material_secret,
    parse_leaf,
    read_state,
    trust_bundle_secret,
)

MINIMUM_CHECKS = 16

OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX = (
    (
        "a_bundle_round_trips_through_its_pem_and_annotation",
        (("issuer-a", "issuer-b"), True, True),
    ),
    (
        "anchors_are_ordered_by_issuer_id_regardless_of_insertion_order",
        (("issuer-a", "issuer-b"), ("issuer-a", "issuer-b")),
    ),
    (
        "re_adding_an_issuer_replaces_its_anchor_rather_than_duplicating_it",
        (1, True),
    ),
    (
        "retaining_keeps_only_the_named_issuers",
        (("issuer-a", "issuer-b", "issuer-c"), ("issuer-a", "issuer-c"), False),
    ),
    (
        "the_material_secret_carries_exactly_the_three_consumer_keys",
        (("ca.crt", "tls.crt", "tls.key"), True, True),
    ),
    (
        "the_trust_only_write_carries_the_bundle_key_alone",
        (("ca.crt",), True),
    ),
    (
        "the_secret_lands_in_the_instances_own_namespace_under_a_purpose_scoped_name",
        ("lumen-system", "lumen-0-serving-tls", "lumen-0-peer-tls"),
    ),
    (
        "the_secret_type_is_opaque_so_trust_can_be_published_before_any_leaf",
        ("Opaque", "Opaque"),
    ),
    (
        "the_labels_name_the_instance_the_manager_and_the_purpose",
        (
            (
                ("app.kubernetes.io/component", "peer-tls"),
                ("app.kubernetes.io/managed-by", "service-k8s"),
                ("app.kubernetes.io/name", "lumen-0"),
            ),
            "serving-tls",
        ),
    ),
    (
        "the_material_secret_annotates_bundle_issuer_and_identity_digest",
        (
            (
                "service-k8s.axiom.dev/identity-digest",
                "service-k8s.axiom.dev/leaf-issuer",
                "service-k8s.axiom.dev/trust-bundle",
            ),
            "issuer-a,issuer-b",
            "issuer-a",
            "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
        ),
    ),
    (
        "a_leaf_parses_to_its_validity_and_a_sha256_of_its_der",
        ("2026-01-01T00:00:00+00:00", "2026-04-01T00:00:00+00:00", 64, False),
    ),
    (
        "read_state_reconstructs_both_the_leaf_and_the_bundle",
        (
            "issuer-a",
            "2026-04-01T00:00:00+00:00",
            "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            ("issuer-a", "issuer-b"),
        ),
    ),
    (
        "the_ready_condition_names_the_issuer_the_leaf_and_the_expiry",
        (
            "ServingCertificateReady",
            "True",
            "Issued",
            "issuer issuer-a; leaf aaaaaaaaaaaaaaaa; expires 2026-04-01T00:00:00+00:00; trusting issuer-a, issuer-b",
        ),
    ),
    (
        "serving_and_peer_conditions_do_not_collide",
        (
            "ServingCertificateReady",
            "ServingCertificateRotating",
            "PeerCertificateReady",
            "PeerCertificateRotating",
        ),
    ),
    (
        "a_rotation_is_reported_on_its_own_condition_with_the_reason_that_triggered_it",
        (
            "True",
            "IssuerRotation",
            "issuing a new peer certificate: the configured issuer changed",
            "False",
            "Stable",
        ),
    ),
    (
        "the_published_fingerprint_is_a_sixteen_character_prefix",
        ("0123456789abcdef", "Renewal", None, True),
    ),
)

SCOPE = InstanceScope(
    namespace="lumen-system", instance="lumen-0", trust_domain="axiom.dev"
)
OWNER = Owner(
    api_version="axiom.dev/v1",
    kind="LumenInstance",
    name="lumen-0",
    uid="11111111-2222-3333-4444-555555555555",
)

ISSUER_A = IssuerId("issuer-a")
ISSUER_B = IssuerId("issuer-b")
ISSUER_C = IssuerId("issuer-c")

ANCHOR_A = "-----BEGIN CERTIFICATE-----\nQUFB\n-----END CERTIFICATE-----"
ANCHOR_B = "-----BEGIN CERTIFICATE-----\nQkJC\n-----END CERTIFICATE-----"
ANCHOR_C = "-----BEGIN CERTIFICATE-----\nQ0ND\n-----END CERTIFICATE-----"

LEAF_PEM = "-----BEGIN CERTIFICATE-----\nbGVhZi1kZXI=\n-----END CERTIFICATE-----"
OTHER_PEM = "-----BEGIN CERTIFICATE-----\nb3RoZXItZGVy\n-----END CERTIFICATE-----"

NOT_BEFORE = datetime(2026, 1, 1, tzinfo=timezone.utc)
NOT_AFTER = datetime(2026, 4, 1, tzinfo=timezone.utc)

IDENTITY_DIGEST = "d" * 64


def reader(der: bytes) -> tuple[datetime, datetime]:
    return (NOT_BEFORE, NOT_AFTER)


BUNDLE_AB = TrustBundle().with_anchor(ISSUER_A, ANCHOR_A).with_anchor(
    ISSUER_B, ANCHOR_B
)

MATERIAL = IssuedMaterial(
    issuer=ISSUER_A,
    certificate_pem=LEAF_PEM,
    chain_pem=ANCHOR_A,
    not_before=NOT_BEFORE,
    not_after=NOT_AFTER,
    fingerprint="f" * 64,
)


def material() -> dict[str, object]:
    return material_secret(
        SCOPE, Purpose.SERVING, OWNER, MATERIAL, "KEYPEM", BUNDLE_AB, IDENTITY_DIGEST
    )


def trust_only() -> dict[str, object]:
    return trust_bundle_secret(SCOPE, Purpose.PEER, OWNER, BUNDLE_AB)


def leaf_data() -> dict[str, bytes]:
    return {
        CERT_KEY: LEAF_PEM.encode("utf-8"),
        TRUST_BUNDLE_KEY: BUNDLE_AB.to_pem().encode("utf-8"),
    }


def leaf_annotations() -> dict[str, str]:
    return {
        TRUST_BUNDLE_ANNOTATION: BUNDLE_AB.annotation(),
        LEAF_ISSUER_ANNOTATION: ISSUER_A.value,
        IDENTITY_DIGEST_ANNOTATION: IDENTITY_DIGEST,
    }


READY_FACTS = CertificateFacts(
    purpose=Purpose.SERVING,
    issuer=ISSUER_A,
    not_after=NOT_AFTER,
    fingerprint="a" * 16,
    trust_bundle=(ISSUER_A, ISSUER_B),
)


def verify_owner_scoped_material_projection_behavior() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. a_bundle_round_trips_through_its_pem_and_annotation
    exp1 = OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[0][1]
    parsed1 = TrustBundle.parse(BUNDLE_AB.to_pem(), BUNDLE_AB.annotation())
    obs1 = (
        tuple(i.value for i in parsed1.issuers()),
        parsed1.to_pem() == BUNDLE_AB.to_pem(),
        parsed1.contains(ISSUER_B),
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. anchors_are_ordered_by_issuer_id_regardless_of_insertion_order
    exp2 = OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[1][1]
    forward2 = TrustBundle().with_anchor(ISSUER_A, ANCHOR_A).with_anchor(
        ISSUER_B, ANCHOR_B
    )
    backward2 = TrustBundle().with_anchor(ISSUER_B, ANCHOR_B).with_anchor(
        ISSUER_A, ANCHOR_A
    )
    obs2 = (
        tuple(i.value for i in forward2.issuers()),
        tuple(i.value for i in backward2.issuers()),
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. re_adding_an_issuer_replaces_its_anchor_rather_than_duplicating_it
    exp3 = OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[2][1]
    replaced3 = TrustBundle().with_anchor(ISSUER_A, ANCHOR_A).with_anchor(
        ISSUER_A, ANCHOR_C
    )
    obs3 = (len(replaced3.entries), replaced3.to_pem() == ANCHOR_C + "\n")
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. retaining_keeps_only_the_named_issuers
    exp4 = OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[3][1]
    three4 = BUNDLE_AB.with_anchor(ISSUER_C, ANCHOR_C)
    kept4 = three4.retaining((ISSUER_A, ISSUER_C))
    obs4 = (
        tuple(i.value for i in three4.issuers()),
        tuple(i.value for i in kept4.issuers()),
        kept4.contains(ISSUER_B),
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. the_material_secret_carries_exactly_the_three_consumer_keys
    exp5 = OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[4][1]
    data5 = material()["stringData"]
    obs5 = (
        tuple(sorted(data5)),
        data5[CERT_KEY] == LEAF_PEM,
        data5[PRIVATE_KEY_KEY] == "KEYPEM",
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. the_trust_only_write_carries_the_bundle_key_alone
    exp6 = OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[5][1]
    data6 = trust_only()["stringData"]
    obs6 = (tuple(sorted(data6)), data6[TRUST_BUNDLE_KEY] == BUNDLE_AB.to_pem())
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. the_secret_lands_in_the_instances_own_namespace_under_a_purpose_scoped_name
    exp7 = OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[6][1]
    obs7 = (
        material()["metadata"]["namespace"],
        material()["metadata"]["name"],
        trust_only()["metadata"]["name"],
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. the_secret_type_is_opaque_so_trust_can_be_published_before_any_leaf
    exp8 = OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[7][1]
    obs8 = (material()["type"], trust_only()["type"])
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. the_labels_name_the_instance_the_manager_and_the_purpose
    exp9 = OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[8][1]
    obs9 = (
        tuple(sorted(trust_only()["metadata"]["labels"].items())),
        material()["metadata"]["labels"]["app.kubernetes.io/component"],
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. the_material_secret_annotates_bundle_issuer_and_identity_digest
    exp10 = OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[9][1]
    ann10 = material()["metadata"]["annotations"]
    obs10 = (
        tuple(sorted(ann10)),
        ann10[TRUST_BUNDLE_ANNOTATION],
        ann10[LEAF_ISSUER_ANNOTATION],
        ann10[IDENTITY_DIGEST_ANNOTATION],
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. a_leaf_parses_to_its_validity_and_a_sha256_of_its_der
    exp11 = OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[10][1]
    facts11 = parse_leaf(LEAF_PEM, reader)
    other11 = parse_leaf(OTHER_PEM, reader)
    obs11 = (
        facts11.not_before.isoformat(),
        facts11.not_after.isoformat(),
        len(facts11.fingerprint),
        facts11.fingerprint == other11.fingerprint,
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. read_state_reconstructs_both_the_leaf_and_the_bundle
    exp12 = OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[11][1]
    state12 = read_state(leaf_data(), leaf_annotations(), reader)
    obs12 = (
        state12.leaf.issuer.value,
        state12.leaf.not_after.isoformat(),
        state12.leaf.identity_digest,
        tuple(i.value for i in state12.bundle.issuers()),
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. the_ready_condition_names_the_issuer_the_leaf_and_the_expiry
    exp13 = OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[12][1]
    ready13, _rotating13 = READY_FACTS.conditions()
    obs13 = (ready13.type_, ready13.status.token, ready13.reason, ready13.message)
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. serving_and_peer_conditions_do_not_collide
    exp14 = OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[13][1]
    serving14 = CertificateFacts(purpose=Purpose.SERVING).conditions()
    peer14 = CertificateFacts(purpose=Purpose.PEER).conditions()
    obs14 = (serving14[0].type_, serving14[1].type_, peer14[0].type_, peer14[1].type_)
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    # 15. a_rotation_is_reported_on_its_own_condition_with_the_reason_that_triggered_it
    exp15 = OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[14][1]
    rotating15 = CertificateFacts(
        purpose=Purpose.PEER, rotating=IssueReason.ISSUER_ROTATION
    ).conditions()[1]
    stable15 = CertificateFacts(purpose=Purpose.PEER).conditions()[1]
    obs15 = (
        rotating15.status.token,
        rotating15.reason,
        rotating15.message,
        stable15.status.token,
        stable15.reason,
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[14][0],
            "expected": exp15,
            "observed": obs15,
            "passed": obs15 == exp15,
        }
    )

    # 16. the_published_fingerprint_is_a_sixteen_character_prefix
    exp16 = OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[15][1]
    from_issue16 = CertificateFacts.from_action(
        Purpose.SERVING,
        ISSUER_A,
        NOT_AFTER,
        "0123456789abcdef" + "z" * 48,
        (ISSUER_A,),
        0,
        Issue(ISSUER_A, IssueReason.RENEWAL),
    )
    from_wait16 = CertificateFacts.from_action(
        Purpose.SERVING,
        ISSUER_A,
        NOT_AFTER,
        None,
        (ISSUER_A,),
        0,
        Wait(NOT_AFTER),
    )
    obs16 = (
        from_issue16.fingerprint,
        from_issue16.rotating.token,
        from_wait16.fingerprint,
        from_wait16.rotating is None,
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_BEHAVIOR_MATRIX[15][0],
            "expected": exp16,
            "observed": obs16,
            "passed": obs16 == exp16,
        }
    )

    return {
        "case_id": "owner-scoped-material-projection-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
