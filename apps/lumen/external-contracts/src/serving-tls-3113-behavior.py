"""EC behavior case for #3113 -- private serving TLS and KSA connect plans.

Every expected value below is an EC-owned literal transcribed from #3113:
R1 requires a named external serving Secret, TLS, and private Service port
7373; R2 requires a supplied private CA and the exact Service DNS/SNI name;
R3 couples verified TLS to the selected KSA token source; R4 keeps trust
handoff ownership separate; and R5 declares the required retained-proof
vocabulary.  Live handshakes, Secret projection, token minting, and GKE
evidence capture are runtime-only and deliberately absent from this model.
"""

from __future__ import annotations

from lumen.serving_tls.admission import (
    decide_authenticated_connect,
    decide_connect_trust,
    decide_serving_tls_plan,
)
from lumen.serving_tls.evidence import required_rows
from lumen.serving_tls.spec import (
    AuthenticatedConnectRequest,
    ConnectTrustRequest,
    ServingTlsRequest,
)
from lumen.serving_tls.verdict import (
    AdmittedAuthenticatedConnect,
    AdmittedConnectTrust,
    AdmittedServingTls,
    Rejection,
)
from lumen.trust_anchor_handoff import classify

MINIMUM_CHECKS = 18

SERVING_TLS_3113_BEHAVIOR_MATRIX = (
    ("named_external_serving_secret_is_admitted", "admitted"),
    ("admitted_serving_plan_keeps_named_secret", "lumen-serving-tls"),
    ("admitted_serving_plan_uses_tls_transport", "https"),
    ("admitted_serving_plan_uses_private_service_port", 7373),
    ("private_ca_and_service_dns_are_admitted", "admitted"),
    ("admitted_trust_plan_keeps_supplied_private_ca", "private-ca.pem"),
    ("admitted_trust_plan_keeps_exact_service_dns_sni", "lumen.lumen-auth.svc"),
    ("verified_tls_with_selected_ksa_is_admitted", "admitted"),
    ("admitted_authenticated_plan_keeps_verified_tls", "verified-tls"),
    ("admitted_authenticated_plan_keeps_selected_ksa_source", "selected-ksa"),
    ("ca_file_is_client_input", "client-input"),
    ("private_trust_is_client_input", "client-input"),
    ("serving_secret_is_external_handoff", "external-handoff"),
    ("public_ca_is_external_handoff", "external-handoff"),
    ("required_evidence_includes_positive_tls_ksa", True),
    ("required_evidence_includes_wrong_name", True),
    ("required_evidence_includes_negative_tls_and_google_token_paths", True),
    ("required_evidence_includes_rotation_redaction_and_cleanup", True),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_serving_tls_3113_behavior() -> dict:
    checks = []

    serving = decide_serving_tls_plan(ServingTlsRequest(
        profile="production", serving_secret="lumen-serving-tls",
        transport="https", service_port=7373,
    ))
    # 1-4. R1 -- production serves only through its named externally supplied
    # Secret, over HTTPS, on the private client Service port.
    obs1 = _outcome(serving); exp1 = SERVING_TLS_3113_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = serving.serving_secret if isinstance(serving, AdmittedServingTls) else "refused"; exp2 = SERVING_TLS_3113_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    obs3 = serving.transport if isinstance(serving, AdmittedServingTls) else "refused"; exp3 = SERVING_TLS_3113_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = serving.service_port if isinstance(serving, AdmittedServingTls) else -1; exp4 = SERVING_TLS_3113_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    trust = decide_connect_trust(ConnectTrustRequest(
        ca_file="private-ca.pem", server_name="lumen.lumen-auth.svc",
    ))
    # 5-7. R2 -- connect uses the supplied private CA for the exact Service SNI.
    obs5 = _outcome(trust); exp5 = SERVING_TLS_3113_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})
    obs6 = trust.ca_file if isinstance(trust, AdmittedConnectTrust) else "refused"; exp6 = SERVING_TLS_3113_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = trust.server_name if isinstance(trust, AdmittedConnectTrust) else "refused"; exp7 = SERVING_TLS_3113_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    authenticated = decide_authenticated_connect(AuthenticatedConnectRequest(
        tls_verification="verified-tls", token_source="selected-ksa",
    ))
    # 8-10. R3 -- only a verified channel carrying the selected KSA is usable.
    obs8 = _outcome(authenticated); exp8 = SERVING_TLS_3113_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    obs9 = authenticated.tls_verification if isinstance(authenticated, AdmittedAuthenticatedConnect) else "refused"; exp9 = SERVING_TLS_3113_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = authenticated.token_source if isinstance(authenticated, AdmittedAuthenticatedConnect) else "refused"; exp10 = SERVING_TLS_3113_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11-14. R4 -- public trust distribution is an external handoff, never a
    # Lumen issuer, publisher, or discovery path.
    obs11 = classify("--ca-file").value; exp11 = SERVING_TLS_3113_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    obs12 = classify("PrivateTrust").value; exp12 = SERVING_TLS_3113_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})
    obs13 = classify("servingTlsSecret").value; exp13 = SERVING_TLS_3113_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    obs14 = classify("public-ca").value; exp14 = SERVING_TLS_3113_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    evidence = frozenset(required_rows())
    # 15-18. R5 -- the pure model owns the proof vocabulary, not live capture.
    obs15 = {"positive-tls-ksa"}.issubset(evidence); exp15 = SERVING_TLS_3113_BEHAVIOR_MATRIX[14][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    obs16 = {"wrong-name"}.issubset(evidence); exp16 = SERVING_TLS_3113_BEHAVIOR_MATRIX[15][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    obs17 = {"unrelated-ca", "missing-ca", "plaintext", "direct-google-token"}.issubset(evidence); exp17 = SERVING_TLS_3113_BEHAVIOR_MATRIX[16][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    obs18 = {"rotation", "redaction", "cleanup"}.issubset(evidence); exp18 = SERVING_TLS_3113_BEHAVIOR_MATRIX[17][1]
    checks.append({"name": SERVING_TLS_3113_BEHAVIOR_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})

    return {"case_id": "serving-tls-3113-behavior", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
