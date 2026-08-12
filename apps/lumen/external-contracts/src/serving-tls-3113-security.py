"""EC security case for #3113 -- fail-closed serving TLS and connect policy.

Matrix literals are transcribed from #3113 R1-R4.  Each refusal observes its
typed reason and named offending field, while an explicit secure neighbour
keeps admission from degenerating into unconditional rejection.  Live TLS,
Kubernetes credentials, and retained GKE artifacts remain runtime-only.
"""

from __future__ import annotations

from lumen.serving_tls.admission import decide_authenticated_connect, decide_connect_trust, decide_serving_tls_plan
from lumen.serving_tls.spec import AuthenticatedConnectRequest, ConnectTrustRequest, ServingTlsRequest
from lumen.serving_tls.verdict import Rejection
from lumen.trust_anchor_handoff import classify

MINIMUM_CHECKS = 23

SERVING_TLS_3113_SECURITY_MATRIX = (
    ("missing_serving_secret_is_refused", "missing_serving_secret"),
    ("missing_serving_secret_refusal_names_secret_field", "serving_secret"),
    ("plaintext_serving_is_refused", "plaintext_transport"),
    ("plaintext_serving_refusal_names_transport_field", "transport"),
    ("secure_named_serving_neighbour_is_admitted", "admitted"),
    ("localhost_sni_is_refused", "unsupported_server_name"),
    ("localhost_sni_refusal_names_server_name_field", "server_name"),
    ("loopback_ip_sni_is_refused", "unsupported_server_name"),
    ("missing_ca_is_refused", "missing_ca_file"),
    ("missing_ca_refusal_names_ca_file_field", "ca_file"),
    ("unrelated_ca_is_refused", "unrelated_ca"),
    ("exact_private_ca_service_dns_neighbour_is_admitted", "admitted"),
    ("unverified_tls_is_refused", "unverified_tls"),
    ("unverified_tls_refusal_names_tls_field", "tls_verification"),
    ("direct_google_token_is_refused", "direct_google_token"),
    ("direct_google_token_refusal_names_token_source_field", "token_source"),
    ("adc_token_is_refused", "direct_google_token"),
    ("gsa_token_is_refused", "direct_google_token"),
    ("metadata_token_is_refused", "direct_google_token"),
    ("verified_tls_selected_ksa_neighbour_is_admitted", "admitted"),
    ("configmap_publisher_is_forbidden", "forbidden-publisher"),
    ("status_discovery_is_forbidden", "forbidden-publisher"),
    ("automatic_ca_discovery_is_forbidden", "forbidden-publisher"),
)


def _reason(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_serving_tls_3113_security() -> dict:
    checks = []

    missing_secret = decide_serving_tls_plan(ServingTlsRequest(profile="production", serving_secret="", transport="https", service_port=7373))
    # 1-2. R1 -- a production caller cannot omit the external serving Secret.
    obs1 = _reason(missing_secret); exp1 = SERVING_TLS_3113_SECURITY_MATRIX[0][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})
    obs2 = missing_secret.field_path if isinstance(missing_secret, Rejection) else ""; exp2 = SERVING_TLS_3113_SECURITY_MATRIX[1][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})
    plaintext = decide_serving_tls_plan(ServingTlsRequest(profile="production", serving_secret="lumen-serving-tls", transport="plaintext", service_port=7373))
    obs3 = _reason(plaintext); exp3 = SERVING_TLS_3113_SECURITY_MATRIX[2][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})
    obs4 = plaintext.field_path if isinstance(plaintext, Rejection) else ""; exp4 = SERVING_TLS_3113_SECURITY_MATRIX[3][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})
    secure_serving = decide_serving_tls_plan(ServingTlsRequest(profile="production", serving_secret="lumen-serving-tls", transport="https", service_port=7373))
    obs5 = _reason(secure_serving); exp5 = SERVING_TLS_3113_SECURITY_MATRIX[4][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    localhost = decide_connect_trust(ConnectTrustRequest(ca_file="private-ca.pem", server_name="localhost"))
    # 6-12. R2 -- only supplied private trust and Service DNS are admissible.
    obs6 = _reason(localhost); exp6 = SERVING_TLS_3113_SECURITY_MATRIX[5][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})
    obs7 = localhost.field_path if isinstance(localhost, Rejection) else ""; exp7 = SERVING_TLS_3113_SECURITY_MATRIX[6][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})
    loopback = decide_connect_trust(ConnectTrustRequest(ca_file="private-ca.pem", server_name="127.0.0.1"))
    obs8 = _reason(loopback); exp8 = SERVING_TLS_3113_SECURITY_MATRIX[7][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})
    missing_ca = decide_connect_trust(ConnectTrustRequest(ca_file="", server_name="lumen.lumen-auth.svc"))
    obs9 = _reason(missing_ca); exp9 = SERVING_TLS_3113_SECURITY_MATRIX[8][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})
    obs10 = missing_ca.field_path if isinstance(missing_ca, Rejection) else ""; exp10 = SERVING_TLS_3113_SECURITY_MATRIX[9][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})
    unrelated_ca = decide_connect_trust(ConnectTrustRequest(ca_file="unrelated-ca.pem", server_name="lumen.lumen-auth.svc"))
    obs11 = _reason(unrelated_ca); exp11 = SERVING_TLS_3113_SECURITY_MATRIX[10][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})
    secure_trust = decide_connect_trust(ConnectTrustRequest(ca_file="private-ca.pem", server_name="lumen.lumen-auth.svc"))
    obs12 = _reason(secure_trust); exp12 = SERVING_TLS_3113_SECURITY_MATRIX[11][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    unverified = decide_authenticated_connect(AuthenticatedConnectRequest(tls_verification="unverified-tls", token_source="selected-ksa"))
    # 13-20. R3 -- verification and the selected KSA are independently enforced.
    obs13 = _reason(unverified); exp13 = SERVING_TLS_3113_SECURITY_MATRIX[12][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})
    obs14 = unverified.field_path if isinstance(unverified, Rejection) else ""; exp14 = SERVING_TLS_3113_SECURITY_MATRIX[13][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})
    google = decide_authenticated_connect(AuthenticatedConnectRequest(tls_verification="verified-tls", token_source="google"))
    obs15 = _reason(google); exp15 = SERVING_TLS_3113_SECURITY_MATRIX[14][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[14][0], "expected": exp15, "observed": obs15, "passed": obs15 == exp15})
    obs16 = google.field_path if isinstance(google, Rejection) else ""; exp16 = SERVING_TLS_3113_SECURITY_MATRIX[15][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[15][0], "expected": exp16, "observed": obs16, "passed": obs16 == exp16})
    adc = decide_authenticated_connect(AuthenticatedConnectRequest(tls_verification="verified-tls", token_source="adc"))
    obs17 = _reason(adc); exp17 = SERVING_TLS_3113_SECURITY_MATRIX[16][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[16][0], "expected": exp17, "observed": obs17, "passed": obs17 == exp17})
    gsa = decide_authenticated_connect(AuthenticatedConnectRequest(tls_verification="verified-tls", token_source="gsa"))
    obs18 = _reason(gsa); exp18 = SERVING_TLS_3113_SECURITY_MATRIX[17][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[17][0], "expected": exp18, "observed": obs18, "passed": obs18 == exp18})
    metadata = decide_authenticated_connect(AuthenticatedConnectRequest(tls_verification="verified-tls", token_source="metadata"))
    obs19 = _reason(metadata); exp19 = SERVING_TLS_3113_SECURITY_MATRIX[18][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[18][0], "expected": exp19, "observed": obs19, "passed": obs19 == exp19})
    secure_auth = decide_authenticated_connect(AuthenticatedConnectRequest(tls_verification="verified-tls", token_source="selected-ksa"))
    obs20 = _reason(secure_auth); exp20 = SERVING_TLS_3113_SECURITY_MATRIX[19][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[19][0], "expected": exp20, "observed": obs20, "passed": obs20 == exp20})

    # 21-23. R4 -- Lumen must not become a public-CA publisher or discoverer.
    obs21 = classify("ConfigMap-publisher").value; exp21 = SERVING_TLS_3113_SECURITY_MATRIX[20][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[20][0], "expected": exp21, "observed": obs21, "passed": obs21 == exp21})
    obs22 = classify("status-discovery").value; exp22 = SERVING_TLS_3113_SECURITY_MATRIX[21][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[21][0], "expected": exp22, "observed": obs22, "passed": obs22 == exp22})
    obs23 = classify("automatic-ca-publication").value; exp23 = SERVING_TLS_3113_SECURITY_MATRIX[22][1]
    checks.append({"name": SERVING_TLS_3113_SECURITY_MATRIX[22][0], "expected": exp23, "observed": obs23, "passed": obs23 == exp23})

    return {"case_id": "serving-tls-3113-security", "minimum_checks": MINIMUM_CHECKS, "checks": checks, "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS}
