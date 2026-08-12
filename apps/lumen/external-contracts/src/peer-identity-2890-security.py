"""EC security case for #2890 -- fail-closed peer identity.

Every expected value below is an EC-owned literal transcribed from #2890:
R4 refuses replicated production without readable complete peer material and
names the offending Secret or key; R7 refuses a replicated production plaintext
default while retaining the explicitly non-mTLS single-replica development
neighbour.  No Kubernetes read, listener, TLS handshake, or Cargo/source claim
is asserted here because those are runtime-only obligations.
"""

from __future__ import annotations

from lumen.peer_identity.admission import decide_profile_peer_tls
from lumen.peer_identity.spec import PeerIdentitySpec, PeerMaterialState, PeerProfile, SecretReference
from lumen.peer_identity.status import decide_peer_identity_status
from lumen.peer_identity.verdict import Rejection

MINIMUM_CHECKS = 14

PEER_IDENTITY_2890_SECURITY_MATRIX = (
    ("replicated_production_without_secret_is_rejected", "peer_tls_secret_required"),
    ("replicated_production_missing_secret_refusal_names_peer_tls_secret", "peer_tls_secret"),
    ("replicated_production_with_explicit_secret_is_admitted", "admitted"),
    ("single_development_no_secret_neighbour_remains_non_mtls", "non_mtls"),
    ("absent_material_reports_peer_identity_not_ready", "False"),
    ("absent_material_uses_stable_reason", "peer_tls_secret_absent"),
    ("absent_material_names_the_configured_secret", "lumen-peer-tls"),
    ("incomplete_material_uses_stable_reason", "peer_tls_secret_incomplete"),
    ("incomplete_material_names_missing_key_field", "tls.key"),
    ("incomplete_material_names_the_secret_reference", "lumen-peer-tls"),
    ("unreadable_material_uses_stable_reason", "peer_tls_secret_unreadable"),
    ("unreadable_material_names_peer_tls_secret_field", "peer_tls_secret"),
    ("unreadable_material_message_names_the_secret", "lumen-peer-tls"),
    ("complete_material_neighbour_reports_peer_identity_ready", "True"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_peer_identity_2890_security() -> dict:
    checks = []
    secret = SecretReference(name="lumen-peer-tls")
    replicated = PeerIdentitySpec(
        profile=PeerProfile.PRODUCTION,
        replicas_per_shard=3,
        peer_tls_secret=secret,
    )

    # 1. R7 -- no implicit replicated-production plaintext configuration exists.
    missing_secret = decide_profile_peer_tls(PeerProfile.PRODUCTION, 3, None)
    obs1 = _outcome(missing_secret)
    exp1 = PEER_IDENTITY_2890_SECURITY_MATRIX[0][1]
    checks.append({"name": PEER_IDENTITY_2890_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R7 -- a useful refusal identifies the field a producer must supply.
    obs2 = missing_secret.field_path if isinstance(missing_secret, Rejection) else ""
    exp2 = PEER_IDENTITY_2890_SECURITY_MATRIX[1][1]
    checks.append({"name": PEER_IDENTITY_2890_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R7 -- the nearest safe production input, an explicit Secret, is admitted.
    explicit_secret = decide_profile_peer_tls(PeerProfile.PRODUCTION, 3, secret)
    obs3 = _outcome(explicit_secret)
    exp3 = PEER_IDENTITY_2890_SECURITY_MATRIX[2][1]
    checks.append({"name": PEER_IDENTITY_2890_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R7 -- the narrow development exception remains an explicit neighbour.
    development = decide_profile_peer_tls(PeerProfile.DEVELOPMENT, 1, None)
    obs4 = development.mode if not isinstance(development, Rejection) else _outcome(development)
    exp4 = PEER_IDENTITY_2890_SECURITY_MATRIX[3][1]
    checks.append({"name": PEER_IDENTITY_2890_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    absent = decide_peer_identity_status(replicated, PeerMaterialState.ABSENT)

    # 5. R4 -- missing material is a false readiness condition, not a warning.
    obs5 = absent.condition.status
    exp5 = PEER_IDENTITY_2890_SECURITY_MATRIX[4][1]
    checks.append({"name": PEER_IDENTITY_2890_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. R4 -- absence has stable vocabulary for controllers and users.
    obs6 = absent.condition.reason
    exp6 = PEER_IDENTITY_2890_SECURITY_MATRIX[5][1]
    checks.append({"name": PEER_IDENTITY_2890_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R4 -- the status says which configured Secret is absent.
    obs7 = next(
        (token for token in absent.condition.message.split() if token == "lumen-peer-tls"),
        "",
    )
    exp7 = PEER_IDENTITY_2890_SECURITY_MATRIX[6][1]
    checks.append({"name": PEER_IDENTITY_2890_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    incomplete = decide_peer_identity_status(replicated, PeerMaterialState.INCOMPLETE)

    # 8. R4 -- incomplete material is not conflated with an absent Secret.
    obs8 = incomplete.condition.reason
    exp8 = PEER_IDENTITY_2890_SECURITY_MATRIX[7][1]
    checks.append({"name": PEER_IDENTITY_2890_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R4 -- the missing credential key is named for repair.
    obs9 = incomplete.condition.field_path
    exp9 = PEER_IDENTITY_2890_SECURITY_MATRIX[8][1]
    checks.append({"name": PEER_IDENTITY_2890_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. R4 -- the incomplete-material explanation also retains the Secret name.
    obs10 = next(
        (token for token in incomplete.condition.message.split() if token == "lumen-peer-tls"),
        "",
    )
    exp10 = PEER_IDENTITY_2890_SECURITY_MATRIX[9][1]
    checks.append({"name": PEER_IDENTITY_2890_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    unreadable = decide_peer_identity_status(replicated, PeerMaterialState.UNREADABLE)

    # 11. R4 -- unreadable material has its own stable, fail-closed reason.
    obs11 = unreadable.condition.reason
    exp11 = PEER_IDENTITY_2890_SECURITY_MATRIX[10][1]
    checks.append({"name": PEER_IDENTITY_2890_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R4 -- the unreadable condition identifies the configured ref field.
    obs12 = unreadable.condition.field_path
    exp12 = PEER_IDENTITY_2890_SECURITY_MATRIX[11][1]
    checks.append({"name": PEER_IDENTITY_2890_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R4 -- the unreadable explanation identifies the affected Secret.
    obs13 = next(
        (token for token in unreadable.condition.message.split() if token == "lumen-peer-tls"),
        "",
    )
    exp13 = PEER_IDENTITY_2890_SECURITY_MATRIX[12][1]
    checks.append({"name": PEER_IDENTITY_2890_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    complete = decide_peer_identity_status(replicated, PeerMaterialState.READABLE)

    # 14. R4 -- readable complete material is the neighbouring ready state.
    obs14 = complete.condition.status
    exp14 = PEER_IDENTITY_2890_SECURITY_MATRIX[13][1]
    checks.append({"name": PEER_IDENTITY_2890_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    return {
        "case_id": "peer-identity-2890-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
