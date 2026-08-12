"""EC security case for #3112 -- fail-closed TLS rotation.

Expected values are EC-owned literals from #3112. R1 refuses shared,
writable, or controller-owned Secret projections. R2 and AC4 retain the last
valid serving generation on invalid material. R3 refuses wrong identity,
untrusted, expired, and plaintext peers. R4 keeps status redacted. R5 keeps
issuer and related ownership surfaces retired and unknown surfaces fail closed.
"""

from __future__ import annotations

from lumen.issuer_ownership import classify
from lumen.tls_rotation.admission import (
    decide_peer_rotation,
    decide_rotation,
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

MINIMUM_CHECKS = 18

TLS_ROTATION_3112_SECURITY_MATRIX = (
    ("shared_serving_and_peer_secret_is_rejected", "secret_names_must_differ"),
    ("shared_secret_refusal_names_the_peer_secret_field", "peer_secret_name"),
    ("distinct_external_read_only_projection_neighbour_is_admitted", "admitted"),
    ("writable_serving_projection_is_rejected", "secret_projection_must_be_read_only"),
    ("controller_owned_peer_projection_is_rejected", "secret_projection_must_be_external"),
    ("invalid_serving_material_is_rejected", "invalid_material"),
    ("serving_refusal_retains_current_generation", 7),
    ("valid_serving_neighbour_is_admitted", "admitted"),
    ("generic_invalid_rotation_is_rejected", "invalid_material"),
    ("generic_invalid_rotation_retains_current_generation", 7),
    ("wrong_identity_peer_is_rejected", "wrong_identity"),
    ("wrong_identity_refusal_names_identity", "identity"),
    ("untrusted_peer_is_rejected", "untrusted"),
    ("expired_peer_is_rejected", "expired"),
    ("plaintext_peer_is_rejected", "plaintext"),
    ("valid_peer_neighbour_is_admitted", 8),
    ("status_snapshot_has_no_sensitive_material_fields", ()),
    ("issuer_surface_is_retired_and_unknown_surface_fails_closed", ("retired-forbidden", "ValueError")),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_tls_rotation_3112_security() -> dict:
    checks = []
    valid_projection = SecretProjectionSpec(
        serving_secret_name="lumen-serving",
        peer_secret_name="lumen-peer",
        serving_read_only=True,
        peer_read_only=True,
        serving_external=True,
        peer_external=True,
    )
    shared = decide_secret_projection(
        SecretProjectionSpec(
            serving_secret_name="lumen-tls",
            peer_secret_name="lumen-tls",
            serving_read_only=True,
            peer_read_only=True,
            serving_external=True,
            peer_external=True,
        )
    )

    # 1. R1 -- serving and peer credentials cannot be the same named Secret.
    obs1 = _outcome(shared)
    exp1 = TLS_ROTATION_3112_SECURITY_MATRIX[0][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R1 -- the rejection tells the caller which supplied identity conflicts.
    obs2 = shared.field_path if isinstance(shared, Rejection) else ""
    exp2 = TLS_ROTATION_3112_SECURITY_MATRIX[1][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R1 -- the nearest valid distinct projection remains admitted.
    obs3 = _outcome(decide_secret_projection(valid_projection))
    exp3 = TLS_ROTATION_3112_SECURITY_MATRIX[2][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    writable = decide_secret_projection(
        SecretProjectionSpec("lumen-serving", "lumen-peer", False, True, True, True)
    )

    # 4. R1 -- explicit writable serving input is refused rather than ignored.
    obs4 = _outcome(writable)
    exp4 = TLS_ROTATION_3112_SECURITY_MATRIX[3][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    owned_peer = decide_secret_projection(
        SecretProjectionSpec("lumen-serving", "lumen-peer", True, True, True, False)
    )

    # 5. R1/R5 -- a controller-owned peer Secret cannot cross this boundary.
    obs5 = _outcome(owned_peer)
    exp5 = TLS_ROTATION_3112_SECURITY_MATRIX[4][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    current = ActiveGeneration(7, "sha256:current", "2029-01-01T00:00:00Z", "ca-current")
    valid = MaterialCandidate(8, "sha256:new", "2030-01-01T00:00:00Z", "lumen-0.lumen-peer", "spiffe://lumen/peer/lumen-0", "ca-next", False)
    invalid = MaterialCandidate(8, "sha256:broken", "2030-01-01T00:00:00Z", "lumen-0.lumen-peer", "spiffe://lumen/peer/lumen-0", "ca-next", False, valid=False)
    serving = decide_serving_rotation(current, invalid)

    # 6. R2 -- serving activation rejects explicitly invalid replacement material.
    obs6 = _outcome(serving)
    exp6 = TLS_ROTATION_3112_SECURITY_MATRIX[5][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R2 -- that serving rejection retains the last known good generation.
    obs7 = serving.retained_generation if isinstance(serving, Rejection) else -1
    exp7 = TLS_ROTATION_3112_SECURITY_MATRIX[6][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R2 -- a neighbouring valid candidate is still activated by this entry point.
    obs8 = _outcome(decide_serving_rotation(current, valid))
    exp8 = TLS_ROTATION_3112_SECURITY_MATRIX[7][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    generic = decide_rotation(current, invalid)

    # 9. AC4 -- the generic decider independently refuses invalid material.
    obs9 = _outcome(generic)
    exp9 = TLS_ROTATION_3112_SECURITY_MATRIX[8][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. AC4 -- and exposes last-known-good retention at its own entry point.
    obs10 = generic.retained_generation if isinstance(generic, Rejection) else -1
    exp10 = TLS_ROTATION_3112_SECURITY_MATRIX[9][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    wrong_identity = decide_peer_rotation(current, MaterialCandidate(8, "sha256:new", "2030-01-01T00:00:00Z", "lumen-0.lumen-peer", "spiffe://other/peer/lumen-0", "ca-next", False))

    # 11. R3 -- a peer certificate with another workload identity is refused.
    obs11 = _outcome(wrong_identity)
    exp11 = TLS_ROTATION_3112_SECURITY_MATRIX[10][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R3 -- identity refusal identifies the named credential dimension.
    obs12 = wrong_identity.field_path if isinstance(wrong_identity, Rejection) else ""
    exp12 = TLS_ROTATION_3112_SECURITY_MATRIX[11][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R3 -- a CA outside the current/overlap trust set is refused.
    obs13 = _outcome(decide_peer_rotation(current, MaterialCandidate(8, "sha256:new", "2030-01-01T00:00:00Z", "lumen-0.lumen-peer", "spiffe://lumen/peer/lumen-0", "ca-unrelated", False)))
    exp13 = TLS_ROTATION_3112_SECURITY_MATRIX[12][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R3 -- an explicitly expired peer candidate is refused.
    obs14 = _outcome(decide_peer_rotation(current, MaterialCandidate(8, "sha256:new", "expired", "lumen-0.lumen-peer", "spiffe://lumen/peer/lumen-0", "ca-next", False)))
    exp14 = TLS_ROTATION_3112_SECURITY_MATRIX[13][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R3 -- plaintext never becomes a rotation fallback.
    obs15 = _outcome(decide_peer_rotation(current, MaterialCandidate(8, "sha256:new", "2030-01-01T00:00:00Z", "lumen-0.lumen-peer", "spiffe://lumen/peer/lumen-0", "ca-next", True)))
    exp15 = TLS_ROTATION_3112_SECURITY_MATRIX[14][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16. R3 -- the valid peer neighbour still reaches the new active generation.
    accepted_peer = decide_peer_rotation(current, valid)
    obs16 = accepted_peer.active_generation if not isinstance(accepted_peer, Rejection) else -1
    exp16 = TLS_ROTATION_3112_SECURITY_MATRIX[15][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    snapshot = derive_status(ReloadState(7, "sha256:current", "2029-01-01T00:00:00Z", 1, 1, "invalid_material", 0, True))

    # 17. R4 -- the public snapshot model cannot carry credential material.
    sensitive = {"private_key", "bearer_token", "certificate_bytes", "filesystem_path"}
    obs17 = tuple(sorted(sensitive.intersection(snapshot.__dataclass_fields__)))
    exp17 = TLS_ROTATION_3112_SECURITY_MATRIX[16][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    # 18. R5 -- issuer ownership is retired and unknown surfaces do not default-admit.
    try:
        classify("unrecognised-issuer-surface")
        unknown = "admitted"
    except ValueError:
        unknown = "ValueError"
    obs18 = (classify("--issuer").value, unknown)
    exp18 = TLS_ROTATION_3112_SECURITY_MATRIX[17][1]
    checks.append({"name": TLS_ROTATION_3112_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    return {
        "case_id": "tls-rotation-3112-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
