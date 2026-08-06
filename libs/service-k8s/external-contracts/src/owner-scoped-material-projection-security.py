from __future__ import annotations

from datetime import datetime, timezone

from service_k8s.application.rotation import IssueReason, IssuerId
from service_k8s.application.status import CertificateFacts, redact
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
    read_state,
    trust_bundle_secret,
)

MINIMUM_CHECKS = 16

OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX = (
    (
        "the_trust_only_write_contains_no_leaf_or_key_material",
        (1, False, False, False),
    ),
    (
        "every_secret_carries_a_controlling_owner_reference_that_blocks_deletion",
        (1, True, True, "11111111-2222-3333-4444-555555555555", True),
    ),
    (
        "a_bundle_whose_annotation_disagrees_with_its_contents_is_emptied_rather_than_guessed_at",
        (True, True),
    ),
    (
        "a_bundle_with_no_annotation_at_all_is_empty",
        (True, True),
    ),
    (
        "an_empty_annotation_with_an_empty_pem_is_a_consistent_empty_bundle",
        (True, "", ""),
    ),
    (
        "a_pem_block_never_survives_into_status",
        ("before [redacted pem] after", False),
    ),
    (
        "a_bearer_token_never_survives_into_status",
        ("call failed: Bearer [redacted] rejected", False),
    ),
    (
        "a_projected_token_never_survives_into_status",
        ("token [redacted token] expired", False),
    ),
    (
        "ordinary_text_passes_through_unharmed",
        (True, "v1.2.3 rolled out"),
    ),
    (
        "a_rotation_does_not_make_the_instance_unready",
        ("True", "Issued", "True", "Renewal"),
    ),
    (
        "a_failing_lifecycle_says_so_rather_than_staying_pending",
        (
            "False",
            "IssuanceFailing",
            "no peer certificate projected after 4 consecutive attempts",
            "Pending",
            "no peer certificate projected yet",
        ),
    ),
    (
        "validity_and_fingerprint_come_from_the_certificate_not_the_annotation",
        ("2026-04-01T00:00:00+00:00", True, "issuer-b"),
    ),
    (
        "an_unreadable_leaf_yields_no_leaf_rather_than_a_partial_one",
        (True, ("issuer-a", "issuer-b")),
    ),
    (
        "a_leaf_missing_either_of_its_claim_annotations_is_not_reconstructed",
        (True, True),
    ),
    (
        "the_facts_type_carries_no_secret_bearing_field",
        (
            "consecutive_failures",
            "fingerprint",
            "issuer",
            "not_after",
            "purpose",
            "rotating",
            "trust_bundle",
        ),
    ),
    (
        "a_non_utf8_bundle_is_empty_rather_than_a_decode_crash",
        (True, True),
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

ANCHOR_A = "-----BEGIN CERTIFICATE-----\nQUFB\n-----END CERTIFICATE-----"
ANCHOR_B = "-----BEGIN CERTIFICATE-----\nQkJC\n-----END CERTIFICATE-----"

LEAF_PEM = "-----BEGIN CERTIFICATE-----\nbGVhZi1kZXI=\n-----END CERTIFICATE-----"

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


JWT = "eyJhbGciOi.eyJzdWIiOi.SflKxwRJSM"
PLAIN = "issuer issuer-a; leaf 0123456789abcdef; expires 2026-04-01T00:00:00+00:00"


def verify_owner_scoped_material_projection_security() -> dict[str, object]:
    checks: list[dict[str, object]] = []

    # 1. the_trust_only_write_contains_no_leaf_or_key_material
    exp1 = OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[0][1]
    data1 = trust_only()["stringData"]
    obs1 = (
        len(data1),
        CERT_KEY in data1,
        PRIVATE_KEY_KEY in data1,
        "KEYPEM" in "".join(data1.values()),
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[0][0],
            "expected": exp1,
            "observed": obs1,
            "passed": obs1 == exp1,
        }
    )

    # 2. every_secret_carries_a_controlling_owner_reference_that_blocks_deletion
    exp2 = OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[1][1]
    ref_m2 = material()["metadata"]["ownerReferences"]
    ref_t2 = trust_only()["metadata"]["ownerReferences"]
    obs2 = (
        len(ref_m2),
        ref_m2[0].get("controller"),
        ref_m2[0].get("blockOwnerDeletion"),
        ref_m2[0].get("uid"),
        ref_t2[0] == ref_m2[0],
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[1][0],
            "expected": exp2,
            "observed": obs2,
            "passed": obs2 == exp2,
        }
    )

    # 3. a_bundle_whose_annotation_disagrees_with_its_contents_is_emptied_rather_than_guessed_at
    exp3 = OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[2][1]
    two_blocks_one_id3 = TrustBundle.parse(BUNDLE_AB.to_pem(), ISSUER_A.value)
    one_block_two_ids3 = TrustBundle.parse(
        ANCHOR_A + "\n", ISSUER_A.value + "," + ISSUER_B.value
    )
    obs3 = (two_blocks_one_id3.is_empty(), one_block_two_ids3.is_empty())
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[2][0],
            "expected": exp3,
            "observed": obs3,
            "passed": obs3 == exp3,
        }
    )

    # 4. a_bundle_with_no_annotation_at_all_is_empty
    exp4 = OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[3][1]
    obs4 = (
        TrustBundle.parse(BUNDLE_AB.to_pem(), None).is_empty(),
        TrustBundle.parse(BUNDLE_AB.to_pem(), "").is_empty(),
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[3][0],
            "expected": exp4,
            "observed": obs4,
            "passed": obs4 == exp4,
        }
    )

    # 5. an_empty_annotation_with_an_empty_pem_is_a_consistent_empty_bundle
    exp5 = OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[4][1]
    consistent5 = TrustBundle.parse("", "")
    obs5 = (consistent5.is_empty(), consistent5.annotation(), consistent5.to_pem())
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[4][0],
            "expected": exp5,
            "observed": obs5,
            "passed": obs5 == exp5,
        }
    )

    # 6. a_pem_block_never_survives_into_status
    exp6 = OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[5][1]
    obs6 = (
        redact("before " + LEAF_PEM + " after"),
        "bGVhZi1kZXI=" in redact("before " + LEAF_PEM + " after"),
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[5][0],
            "expected": exp6,
            "observed": obs6,
            "passed": obs6 == exp6,
        }
    )

    # 7. a_bearer_token_never_survives_into_status
    exp7 = OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[6][1]
    obs7 = (
        redact("call failed: Bearer sk-abcdef123456 rejected"),
        "sk-abcdef123456" in redact("Bearer sk-abcdef123456"),
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[6][0],
            "expected": exp7,
            "observed": obs7,
            "passed": obs7 == exp7,
        }
    )

    # 8. a_projected_token_never_survives_into_status
    exp8 = OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[7][1]
    obs8 = (redact("token " + JWT + " expired"), JWT in redact(JWT))
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[7][0],
            "expected": exp8,
            "observed": obs8,
            "passed": obs8 == exp8,
        }
    )

    # 9. ordinary_text_passes_through_unharmed
    exp9 = OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[8][1]
    obs9 = (redact(PLAIN) == PLAIN, redact("v1.2.3 rolled out"))
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[8][0],
            "expected": exp9,
            "observed": obs9,
            "passed": obs9 == exp9,
        }
    )

    # 10. a_rotation_does_not_make_the_instance_unready
    exp10 = OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[9][1]
    rotating_ready10 = CertificateFacts(
        purpose=Purpose.SERVING,
        issuer=ISSUER_A,
        not_after=NOT_AFTER,
        fingerprint="a" * 16,
        rotating=IssueReason.RENEWAL,
    ).conditions()
    obs10 = (
        rotating_ready10[0].status.token,
        rotating_ready10[0].reason,
        rotating_ready10[1].status.token,
        rotating_ready10[1].reason,
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[9][0],
            "expected": exp10,
            "observed": obs10,
            "passed": obs10 == exp10,
        }
    )

    # 11. a_failing_lifecycle_says_so_rather_than_staying_pending
    exp11 = OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[10][1]
    failing11 = CertificateFacts(
        purpose=Purpose.PEER, consecutive_failures=4
    ).conditions()[0]
    pending11 = CertificateFacts(purpose=Purpose.PEER).conditions()[0]
    obs11 = (
        failing11.status.token,
        failing11.reason,
        failing11.message,
        pending11.reason,
        pending11.message,
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[10][0],
            "expected": exp11,
            "observed": obs11,
            "passed": obs11 == exp11,
        }
    )

    # 12. validity_and_fingerprint_come_from_the_certificate_not_the_annotation
    exp12 = OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[11][1]
    lying12 = dict(leaf_annotations())
    lying12[LEAF_ISSUER_ANNOTATION] = ISSUER_B.value
    lied12 = read_state(leaf_data(), lying12, reader)
    honest12 = read_state(leaf_data(), leaf_annotations(), reader)
    obs12 = (
        lied12.leaf.not_after.isoformat(),
        lied12.leaf.fingerprint == honest12.leaf.fingerprint,
        lied12.leaf.issuer.value,
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[11][0],
            "expected": exp12,
            "observed": obs12,
            "passed": obs12 == exp12,
        }
    )

    # 13. an_unreadable_leaf_yields_no_leaf_rather_than_a_partial_one
    exp13 = OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[12][1]
    broken13 = dict(leaf_data())
    broken13[CERT_KEY] = b"not a certificate at all"
    state13 = read_state(broken13, leaf_annotations(), reader)
    obs13 = (state13.leaf is None, tuple(i.value for i in state13.bundle.issuers()))
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[12][0],
            "expected": exp13,
            "observed": obs13,
            "passed": obs13 == exp13,
        }
    )

    # 14. a_leaf_missing_either_of_its_claim_annotations_is_not_reconstructed
    exp14 = OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[13][1]
    no_issuer14 = dict(leaf_annotations())
    del no_issuer14[LEAF_ISSUER_ANNOTATION]
    no_digest14 = dict(leaf_annotations())
    del no_digest14[IDENTITY_DIGEST_ANNOTATION]
    obs14 = (
        read_state(leaf_data(), no_issuer14, reader).leaf is None,
        read_state(leaf_data(), no_digest14, reader).leaf is None,
    )
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[13][0],
            "expected": exp14,
            "observed": obs14,
            "passed": obs14 == exp14,
        }
    )

    # 15. the_facts_type_carries_no_secret_bearing_field
    exp15 = OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[14][1]
    obs15 = tuple(sorted(CertificateFacts.__annotations__))
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[14][0],
            "expected": exp15,
            "observed": obs15,
            "passed": obs15 == exp15,
        }
    )

    # 16. a_non_utf8_bundle_is_empty_rather_than_a_decode_crash
    exp16 = OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[15][1]
    undecodable16 = dict(leaf_data())
    undecodable16[TRUST_BUNDLE_KEY] = b"\xff\xfe not utf-8"
    state16 = read_state(undecodable16, leaf_annotations(), reader)
    obs16 = (state16.bundle.is_empty(), state16.leaf is not None)
    checks.append(
        {
            "name": OWNER_SCOPED_MATERIAL_PROJECTION_SECURITY_MATRIX[15][0],
            "expected": exp16,
            "observed": obs16,
            "passed": obs16 == exp16,
        }
    )

    return {
        "case_id": "owner-scoped-material-projection-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
