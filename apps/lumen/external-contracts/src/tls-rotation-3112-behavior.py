"""EC behavior case for #3112 -- externally provisioned TLS rotation.

Every expected value in this matrix is an EC-owned literal from #3112: R1
requires distinct external read-only Secret projections; R2 activates a valid
serving generation; R3 retains prior peer trust through an admitted overlap;
R4 exposes a bounded redacted snapshot; and R5 keeps serving and peer
credentials at the settled external-secret ownership boundary.
"""

from __future__ import annotations

from lumen.issuer_ownership import classify
from lumen.tls_rotation.admission import (
    decide_peer_rotation,
    decide_secret_projection,
    decide_serving_rotation,
)
from lumen.tls_rotation.spec import (
    ActiveGeneration,
    MaterialCandidate,
    ReloadState,
    SecretProjectionSpec,
)
from lumen.tls_rotation.status import derive_status
from lumen.tls_rotation.verdict import Rejection

MINIMUM_CHECKS = 9

TLS_ROTATION_3112_BEHAVIOR_MATRIX = (
    ("distinct_external_read_only_secrets_are_admitted", "admitted"),
    ("admitted_projection_preserves_both_named_secrets", ("lumen-serving", "lumen-peer")),
    ("valid_serving_material_activates_its_next_generation", 8),
    ("valid_peer_material_activates_its_next_generation", 8),
    ("peer_activation_retains_prior_trust_during_overlap", "ca-current"),
    ("status_exposes_the_bounded_rotation_snapshot", (8, "sha256:new", "2030-01-01T00:00:00Z", 4, 1, "invalid_material", 1)),
    ("status_exposes_the_serving_predicate_value", "serving"),
    ("serving_secret_is_classified_external", "external-secret"),
    ("peer_secret_is_classified_external", "external-secret"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_tls_rotation_3112_behavior() -> dict:
    checks = []

    projection = decide_secret_projection(
        SecretProjectionSpec(
            serving_secret_name="lumen-serving",
            peer_secret_name="lumen-peer",
            serving_read_only=True,
            peer_read_only=True,
            serving_external=True,
            peer_external=True,
        )
    )

    # 1. R1 -- distinct externally supplied, read-only projections are the
    #    normal configuration; fail-closed admission must not reject it.
    obs1 = _outcome(projection)
    exp1 = TLS_ROTATION_3112_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": TLS_ROTATION_3112_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- admission carries both independently named Secret identities.
    obs2 = (projection.serving_secret_name, projection.peer_secret_name) if not isinstance(projection, Rejection) else ()
    exp2 = TLS_ROTATION_3112_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": TLS_ROTATION_3112_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    current = ActiveGeneration(
        generation=7,
        fingerprint="sha256:current",
        expires_at="2029-01-01T00:00:00Z",
        trust_fingerprint="ca-current",
    )
    candidate = MaterialCandidate(
        generation=8,
        fingerprint="sha256:new",
        expires_at="2030-01-01T00:00:00Z",
        dns_name="lumen-0.lumen-peer",
        identity="spiffe://lumen/peer/lumen-0",
        ca_fingerprint="ca-next",
        plaintext=False,
    )

    serving = decide_serving_rotation(current, candidate)

    # 3. R2 -- a valid replacement activates the supplied next generation.
    obs3 = serving.generation if not isinstance(serving, Rejection) else -1
    exp3 = TLS_ROTATION_3112_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": TLS_ROTATION_3112_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    peer = decide_peer_rotation(current, candidate)

    # 4. R3 -- an admitted peer replacement has the same new active generation.
    obs4 = peer.active_generation if not isinstance(peer, Rejection) else -1
    exp4 = TLS_ROTATION_3112_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": TLS_ROTATION_3112_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R3 -- peer authentication retains the previous trust anchor while
    #    the candidate anchor overlaps it.
    obs5 = peer.retained_trust_fingerprint if not isinstance(peer, Rejection) else ""
    exp5 = TLS_ROTATION_3112_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": TLS_ROTATION_3112_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    snapshot = derive_status(
        ReloadState(
            active_generation=8,
            fingerprint="sha256:new",
            expires_at="2030-01-01T00:00:00Z",
            accepted_count=4,
            rejected_count=1,
            failure_reason="invalid_material",
            overlap_count=1,
            serving=True,
        )
    )

    # 6. R4 -- generation, safe fingerprint, expiry and bounded counters/reason
    #    are all independently useful to an operator.
    obs6 = (snapshot.generation, snapshot.fingerprint, snapshot.expires_at, snapshot.accepted_count, snapshot.rejected_count, snapshot.failure_reason, snapshot.overlap_count)
    exp6 = TLS_ROTATION_3112_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": TLS_ROTATION_3112_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R4 -- serving is an explicit status value, not inferred from a log.
    obs7 = "serving" if snapshot.serving else "not_serving"
    exp7 = TLS_ROTATION_3112_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": TLS_ROTATION_3112_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R5 -- serving material remains an operator input, not Lumen-owned issuance.
    obs8 = classify("servingTlsSecret").value
    exp8 = TLS_ROTATION_3112_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": TLS_ROTATION_3112_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R5 -- peer material is independently pinned to the same boundary.
    obs9 = classify("peerTlsSecret").value
    exp9 = TLS_ROTATION_3112_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": TLS_ROTATION_3112_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    return {
        "case_id": "tls-rotation-3112-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
