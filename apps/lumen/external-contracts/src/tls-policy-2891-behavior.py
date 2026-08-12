"""EC behavior case for #2891 -- externally owned serving and peer TLS.

Every expected value in this matrix is an EC-owned literal from #2891: R2
requires externally provisioned serving and peer Secrets, R3 requires the
admitted client-KSA/SAR serving path, R4 requires loopback forwarding,
Service-DNS verification, and an audience-bound KSA token, R5 admits trusted
peer mTLS before Raft routing, R6 requires coupled external generations and
changed active fingerprints, and R8 retains only the stated public evidence
metadata.  Runtime-only cluster, handshake, and process claims are deliberately
not represented here.
"""

from __future__ import annotations

from lumen.issuer_ownership import classify
from lumen.topology.evidence_policy import EvidenceField, classify_field
from lumen.topology.tls_policy import (
    ConnectRequest,
    ExternalRotation,
    PeerCase,
    ServingCase,
    decide_connect_policy,
    decide_external_rotation,
    decide_peer_case,
    decide_serving_case,
)
from lumen.topology.tls_verdict import Refusal

MINIMUM_CHECKS = 21

TLS_POLICY_2891_BEHAVIOR_MATRIX = (
    ("serving_tls_secret_is_externally_owned", "external-secret"),
    ("peer_tls_secret_is_externally_owned", "external-secret"),
    ("client_ksa_with_allowed_sar_is_admitted", "admitted"),
    ("admitted_serving_case_names_client_ksa_subject", "system:serviceaccount:lumen-auth:client"),
    ("admitted_serving_case_records_allowed_sar", "allowed"),
    ("connect_binds_port_forward_to_loopback", "127.0.0.1"),
    ("connect_verifies_private_service_dns_identity", "lumen.lumen-auth.svc"),
    ("connect_uses_audience_bound_client_ksa_token", "lumen.axiom.dev"),
    ("trusted_peer_mtls_is_admitted", "admitted"),
    ("trusted_peer_mtls_reaches_raft_router", True),
    ("external_rotation_requires_both_secret_generations", "serving-and-peer"),
    ("external_rotation_changes_active_serving_fingerprint", True),
    ("external_rotation_changes_active_peer_fingerprint", True),
    ("run_id_is_retained_evidence", "retained"),
    ("public_fingerprint_is_retained_evidence", "retained"),
    ("assertions_are_retained_evidence", "retained"),
    ("cluster_image_identity_is_retained_evidence", "retained"),
    ("ksa_subjects_are_retained_evidence", "retained"),
    ("public_generations_are_retained_evidence", "retained"),
    ("timings_are_retained_evidence", "retained"),
    ("cleanup_is_retained_evidence", "retained"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Refusal) else "admitted"


def verify_tls_policy_2891_behavior() -> dict:
    checks = []

    # 1-2. R2 -- the operator consumes, rather than issues, both TLS inputs.
    obs1 = classify("servingTlsSecret").value
    exp1 = TLS_POLICY_2891_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = classify("peerTlsSecret").value
    exp2 = TLS_POLICY_2891_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    serving = decide_serving_case(ServingCase(
        credential_kind="client-ksa", subject="system:serviceaccount:lumen-auth:client",
        subject_access_review="allowed", transport="tls", ca="service-ca",
        service_dns="lumen.lumen-auth.svc",
    ))
    # 3-5. R3 -- the named permitted KSA/SAR path remains usable and explicit.
    obs3 = _outcome(serving)
    exp3 = TLS_POLICY_2891_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = serving.subject if not isinstance(serving, Refusal) else "refused"
    exp4 = TLS_POLICY_2891_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = serving.subject_access_review if not isinstance(serving, Refusal) else "refused"
    exp5 = TLS_POLICY_2891_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    connect = decide_connect_policy(ConnectRequest(
        forward_host="127.0.0.1", service_dns="lumen.lumen-auth.svc",
        token_subject="system:serviceaccount:lumen-auth:client", token_audience="lumen.axiom.dev",
        insecure_skip_hostname_verification=False,
    ))
    # 6-8. R4 -- every value required for the CLI path is observable in its plan.
    obs6 = connect.forward_host if not isinstance(connect, Refusal) else "refused"
    exp6 = TLS_POLICY_2891_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = connect.server_name if not isinstance(connect, Refusal) else "refused"
    exp7 = TLS_POLICY_2891_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    obs8 = connect.token_audience if not isinstance(connect, Refusal) else "refused"
    exp8 = TLS_POLICY_2891_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    peer = decide_peer_case(PeerCase(
        credential_kind="peer-mtls", transport="tls", ca="peer-ca", instance="lumen-a",
        certificate_identity="lumen-a", port=7374,
    ))
    # 9-10. R5 -- trusted peer credentials are admitted to the peer router.
    obs9 = _outcome(peer)
    exp9 = TLS_POLICY_2891_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = peer.raft_router_reachable if not isinstance(peer, Refusal) else False
    exp10 = TLS_POLICY_2891_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    rotation = decide_external_rotation(ExternalRotation(
        serving_generation="serving-v2", peer_generation="peer-v2",
        previous_serving_fingerprint="serving-fp-v1", active_serving_fingerprint="serving-fp-v2",
        previous_peer_fingerprint="peer-fp-v1", active_peer_fingerprint="peer-fp-v2",
    ))
    # 11-13. R6 -- both externally owned generations rotate and activate together.
    obs11 = rotation.required_generations if not isinstance(rotation, Refusal) else "refused"
    exp11 = TLS_POLICY_2891_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = rotation.serving_fingerprint_changed if not isinstance(rotation, Refusal) else False
    exp12 = TLS_POLICY_2891_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    obs13 = rotation.peer_fingerprint_changed if not isinstance(rotation, Refusal) else False
    exp13 = TLS_POLICY_2891_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14-16. R8 -- allowed evidence is a closed public-metadata vocabulary.
    obs14 = classify_field(EvidenceField("run_id")).value
    exp14 = TLS_POLICY_2891_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    obs15 = classify_field(EvidenceField("public_fingerprint")).value
    exp15 = TLS_POLICY_2891_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    obs16 = classify_field(EvidenceField("assertions")).value
    exp16 = TLS_POLICY_2891_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    # 17. R8 -- cluster/image identity is public run provenance.
    obs17 = classify_field(EvidenceField("cluster_image_identity")).value
    exp17 = TLS_POLICY_2891_BEHAVIOR_MATRIX[16][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    # 18. R8 -- the KSA subject identifies the public workload principal.
    obs18 = classify_field(EvidenceField("ksa_subjects")).value
    exp18 = TLS_POLICY_2891_BEHAVIOR_MATRIX[17][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    # 19. R8 -- a generation number, like its fingerprint, is public metadata.
    obs19 = classify_field(EvidenceField("public_generations")).value
    exp19 = TLS_POLICY_2891_BEHAVIOR_MATRIX[18][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    # 20. R8 -- bounded timings are retained assertions, not credentials.
    obs20 = classify_field(EvidenceField("timings")).value
    exp20 = TLS_POLICY_2891_BEHAVIOR_MATRIX[19][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})

    # 21. R8 -- cleanup outcome is retained so evidence can show finalization.
    obs21 = classify_field(EvidenceField("cleanup")).value
    exp21 = TLS_POLICY_2891_BEHAVIOR_MATRIX[20][1]
    checks.append({"name": TLS_POLICY_2891_BEHAVIOR_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})

    return {"case_id": "tls-policy-2891-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
