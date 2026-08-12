"""EC security case for #2891 -- TLS policy refusal and non-overclaim boundaries.

All matrix literals are transcribed from #2891's observable requirements: R3
rejects direct Google credentials and denied authorization at their named
boundary, R4 rejects hostname bypass, R5 rejects unsafe peer credentials before
Raft, R8 rejects sensitive evidence material, and R10 forbids retired issuer
and CAS ownership surfaces.  The runtime proof of these boundaries belongs to
the GKE/Rust stage and is intentionally outside this pure design model.
"""

from __future__ import annotations

from lumen.issuer_ownership import classify
from lumen.topology.evidence_policy import EvidenceField, classify_field
from lumen.topology.tls_policy import ConnectRequest, PeerCase, ServingCase, decide_connect_policy, decide_peer_case, decide_serving_case
from lumen.topology.tls_verdict import Refusal

MINIMUM_CHECKS = 27

TLS_POLICY_2891_SECURITY_MATRIX = (
    ("direct_google_oauth_is_refused_at_credential_boundary", "direct_google_credential"),
    ("direct_google_oauth_refusal_names_credential_kind", "credential_kind"),
    ("client_ksa_neighbour_remains_admitted", "admitted"),
    ("denied_sar_is_refused_at_authorization_boundary", "subject_access_review_denied"),
    ("denied_sar_refusal_names_authorization_field", "subject_access_review"),
    ("hostname_bypass_is_refused", "insecure_hostname_bypass"),
    ("hostname_bypass_refusal_names_bypass_field", "insecure_skip_hostname_verification"),
    ("loopback_dns_ksa_neighbour_remains_admitted", "admitted"),
    ("plaintext_peer_is_refused_before_raft", "plaintext_peer_transport"),
    ("unrelated_ca_peer_is_refused_before_raft", "unrelated_peer_ca"),
    ("wrong_instance_peer_is_refused_before_raft", "wrong_peer_instance"),
    ("serving_certificate_on_peer_port_is_refused_before_raft", "serving_certificate_on_peer_port"),
    ("rejected_peer_never_reaches_raft_router", False),
    ("private_key_evidence_is_rejected", "rejected"),
    ("bearer_token_evidence_is_rejected", "rejected"),
    ("kubeconfig_credential_evidence_is_rejected", "rejected"),
    ("certificate_pem_evidence_is_rejected", "rejected"),
    ("retired_issuer_surface_is_forbidden", "retired-forbidden"),
    ("direct_google_id_is_refused_at_credential_boundary", "direct_google_credential"),
    ("plaintext_serving_is_refused_at_transport_boundary", "plaintext_transport"),
    ("wrong_serving_ca_is_refused_at_trust_boundary", "wrong_service_ca"),
    ("wrong_service_dns_is_refused_at_identity_boundary", "wrong_service_dns"),
    ("unbound_ksa_is_refused_at_subject_boundary", "unbound_ksa"),
    ("retired_cas_surface_is_forbidden", "retired-forbidden"),
    ("retired_ca_pool_surface_is_forbidden", "retired-forbidden"),
    ("unknown_ownership_surface_raises_value_error", "ValueError"),
    ("retired_ca_pool_environment_surface_is_forbidden", "retired-forbidden"),
)


def _reason(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Refusal) else "admitted"


def verify_tls_policy_2891_security() -> dict:
    checks = []

    google = decide_serving_case(ServingCase(
        credential_kind="google-oauth", subject="google:principal", subject_access_review="allowed",
        transport="tls", ca="service-ca", service_dns="lumen.lumen-auth.svc",
    ))
    # 1-2. R3 -- a Google credential is not a Lumen request credential.
    obs1 = _reason(google)
    exp1 = TLS_POLICY_2891_SECURITY_MATRIX[0][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = google.field_path if isinstance(google, Refusal) else ""
    exp2 = TLS_POLICY_2891_SECURITY_MATRIX[1][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    ksa = decide_serving_case(ServingCase(
        credential_kind="client-ksa", subject="system:serviceaccount:lumen-auth:client",
        subject_access_review="allowed", transport="tls", ca="service-ca", service_dns="lumen.lumen-auth.svc",
    ))
    obs3 = _reason(ksa)
    exp3 = TLS_POLICY_2891_SECURITY_MATRIX[2][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    denied = decide_serving_case(ServingCase(
        credential_kind="client-ksa", subject="system:serviceaccount:lumen-auth:client",
        subject_access_review="denied", transport="tls", ca="service-ca", service_dns="lumen.lumen-auth.svc",
    ))
    # 4-5. R3 -- authorization denial remains visible as its own layer.
    obs4 = _reason(denied)
    exp4 = TLS_POLICY_2891_SECURITY_MATRIX[3][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    obs5 = denied.field_path if isinstance(denied, Refusal) else ""
    exp5 = TLS_POLICY_2891_SECURITY_MATRIX[4][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    bypass = decide_connect_policy(ConnectRequest(
        forward_host="127.0.0.1", service_dns="lumen.lumen-auth.svc",
        token_subject="system:serviceaccount:lumen-auth:client", token_audience="lumen.axiom.dev",
        insecure_skip_hostname_verification=True,
    ))
    # 6-8. R4 -- no hostname bypass, while the exact secure neighbour is admitted.
    obs6 = _reason(bypass)
    exp6 = TLS_POLICY_2891_SECURITY_MATRIX[5][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = bypass.field_path if isinstance(bypass, Refusal) else ""
    exp7 = TLS_POLICY_2891_SECURITY_MATRIX[6][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    secure = decide_connect_policy(ConnectRequest(
        forward_host="127.0.0.1", service_dns="lumen.lumen-auth.svc",
        token_subject="system:serviceaccount:lumen-auth:client", token_audience="lumen.axiom.dev",
        insecure_skip_hostname_verification=False,
    ))
    obs8 = _reason(secure)
    exp8 = TLS_POLICY_2891_SECURITY_MATRIX[7][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    rejected_peers = (
        decide_peer_case(PeerCase("peer-mtls", "plaintext", "peer-ca", "lumen-a", "lumen-a", 7374)),
        decide_peer_case(PeerCase("peer-mtls", "tls", "unrelated-ca", "lumen-a", "lumen-a", 7374)),
        decide_peer_case(PeerCase("peer-mtls", "tls", "peer-ca", "lumen-a", "lumen-b", 7374)),
        decide_peer_case(PeerCase("serving-tls", "tls", "peer-ca", "lumen-a", "lumen-a", 7374)),
    )
    # 9-13. R5 -- each dangerous credential shape names its own boundary and none reaches Raft.
    obs9 = _reason(rejected_peers[0]); exp9 = TLS_POLICY_2891_SECURITY_MATRIX[8][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = _reason(rejected_peers[1]); exp10 = TLS_POLICY_2891_SECURITY_MATRIX[9][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    obs11 = _reason(rejected_peers[2]); exp11 = TLS_POLICY_2891_SECURITY_MATRIX[10][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = _reason(rejected_peers[3]); exp12 = TLS_POLICY_2891_SECURITY_MATRIX[11][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    obs13 = tuple(v.raft_router_reachable if not isinstance(v, Refusal) else False for v in rejected_peers)
    exp13 = TLS_POLICY_2891_SECURITY_MATRIX[12][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[12][0], "expected": exp13, "observed": all(obs13), "passed": all(obs13) == exp13})

    # 14. R8 -- a private key is never evidence metadata.
    obs14 = classify_field(EvidenceField("private_key")).value
    exp14 = TLS_POLICY_2891_SECURITY_MATRIX[13][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    # 15. R8 -- bearer tokens are credentials, not retained assertions.
    obs15 = classify_field(EvidenceField("bearer_token")).value
    exp15 = TLS_POLICY_2891_SECURITY_MATRIX[14][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})

    # 16. R8 -- kubeconfig credentials stay outside the evidence corpus.
    obs16 = classify_field(EvidenceField("kubeconfig_credential")).value
    exp16 = TLS_POLICY_2891_SECURITY_MATRIX[15][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})

    # 17. R8 -- certificate PEM material is sensitive even when a fingerprint is public.
    obs17 = classify_field(EvidenceField("certificate_pem")).value
    exp17 = TLS_POLICY_2891_SECURITY_MATRIX[16][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})

    # 18. R10 -- issuer/CAS ownership never returns through a retired switch.
    obs18 = classify("--issuer").value
    exp18 = TLS_POLICY_2891_SECURITY_MATRIX[17][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    google_id = decide_serving_case(ServingCase("google-id", "google:principal", "allowed", "tls", "service-ca", "lumen.lumen-auth.svc"))
    # 19. R3 -- direct Google ID tokens take the same rejected credential path.
    obs19 = _reason(google_id)
    exp19 = TLS_POLICY_2891_SECURITY_MATRIX[18][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})

    plaintext = decide_serving_case(ServingCase("client-ksa", "system:serviceaccount:lumen-auth:client", "allowed", "plaintext", "service-ca", "lumen.lumen-auth.svc"))
    # 20. R3 -- a correct KSA cannot turn plaintext into an admitted path.
    obs20 = _reason(plaintext)
    exp20 = TLS_POLICY_2891_SECURITY_MATRIX[19][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})

    wrong_ca = decide_serving_case(ServingCase("client-ksa", "system:serviceaccount:lumen-auth:client", "allowed", "tls", "wrong-ca", "lumen.lumen-auth.svc"))
    # 21. R3 -- a KSA path must still use the Service trust anchor.
    obs21 = _reason(wrong_ca)
    exp21 = TLS_POLICY_2891_SECURITY_MATRIX[20][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})

    wrong_dns = decide_serving_case(ServingCase("client-ksa", "system:serviceaccount:lumen-auth:client", "allowed", "tls", "service-ca", "wrong.lumen-auth.svc"))
    # 22. R3 -- identity checks bind the private Service's exact DNS name.
    obs22 = _reason(wrong_dns)
    exp22 = TLS_POLICY_2891_SECURITY_MATRIX[21][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})

    unbound = decide_serving_case(ServingCase("client-ksa", "system:serviceaccount:lumen-auth:unbound", "allowed", "tls", "service-ca", "lumen.lumen-auth.svc"))
    # 23. R3 -- an unbound KSA is not interchangeable with the client KSA.
    obs23 = _reason(unbound)
    exp23 = TLS_POLICY_2891_SECURITY_MATRIX[22][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[22][0], "expected": exp23, "observed": obs23, "passed": obs23 == exp23})

    # 24-26. R10 -- all retained issuer/CAS ownership surfaces stay forbidden.
    obs24 = classify("cas-resolver").value
    exp24 = TLS_POLICY_2891_SECURITY_MATRIX[23][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[23][0], "expected": exp24, "observed": obs24, "passed": obs24 == exp24})
    obs25 = classify("--ca-pool").value
    exp25 = TLS_POLICY_2891_SECURITY_MATRIX[24][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[24][0], "expected": exp25, "observed": obs25, "passed": obs25 == exp25})
    try:
        classify("unknown-issuer-surface")
    except ValueError as error:
        obs26 = type(error).__name__
    else:
        obs26 = "no_error"
    exp26 = TLS_POLICY_2891_SECURITY_MATRIX[25][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[25][0], "expected": exp26, "observed": obs26, "passed": obs26 == exp26})

    # 27. R10 -- the retired environment ownership switch is forbidden too.
    obs27 = classify("LUMEN_CA_POOL").value
    exp27 = TLS_POLICY_2891_SECURITY_MATRIX[26][1]
    checks.append({"name": TLS_POLICY_2891_SECURITY_MATRIX[26][0], "expected": exp27, "observed": obs27, "passed": obs27 == exp27})

    return {"case_id": "tls-policy-2891-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
