"""EC behavior case for #2890 -- pure peer-identity plans and policies.

Every expected value below is an EC-owned literal transcribed from #2890:
R1/R2 require the three-key read-only Secret projection and its four exact
peer-TLS bindings; R3 requires the dedicated HTTPS peer listener on 7374; R4
names the explicit no-peer status for a single member; R5 carries the supplied
DNS identity, trust domain, and mutual client verification; and R7 permits an
explicit single-replica development non-mTLS mode.
"""

from __future__ import annotations

from lumen.peer_identity.admission import (
    decide_profile_peer_tls,
    decide_raft_transport,
    peer_identity_expectation,
)
from lumen.peer_identity.projection import decide_peer_tls_projection
from lumen.peer_identity.spec import PeerIdentitySpec, PeerMaterialState, PeerProfile, SecretReference
from lumen.peer_identity.status import decide_peer_identity_status
from lumen.peer_identity.verdict import Rejection

MINIMUM_CHECKS = 14

PEER_IDENTITY_2890_BEHAVIOR_MATRIX = (
    ("configured_secret_uses_exact_peer_tls_key_tuple", ("tls.crt", "tls.key", "ca.crt")),
    ("configured_secret_mount_uses_read_only_access", "read_only"),
    ("configured_secret_uses_the_lumen_peer_mount", "/var/run/lumen/peer-tls"),
    (
        "configured_secret_exports_exact_peer_tls_environment",
        (
            ("LUMEN_PEER_MTLS", "on"),
            ("LUMEN_PEER_TLS_CERT", "/var/run/lumen/peer-tls/tls.crt"),
            ("LUMEN_PEER_TLS_KEY", "/var/run/lumen/peer-tls/tls.key"),
            ("LUMEN_PEER_TLS_CA", "/var/run/lumen/peer-tls/ca.crt"),
        ),
    ),
    ("absent_secret_has_no_peer_projection", None),
    ("replicated_transport_uses_mutual_tls_mode", "mtls"),
    ("replicated_transport_uses_https", "https"),
    ("replicated_transport_uses_dedicated_port_7374", 7374),
    ("single_member_transport_is_explicitly_non_peer", "no_peer"),
    ("expectation_carries_supplied_server_dns", "lumen-orders-0.lumen-orders.default.svc.cluster.local"),
    ("expectation_carries_supplied_instance_trust_domain", "lumen.axiom.dev"),
    ("expectation_requires_mutual_client_certificate_verification", "required"),
    ("single_replica_development_is_explicit_non_mtls", "non_mtls"),
    ("single_member_status_is_explicitly_no_peer", "peer_identity_not_required"),
)


def _outcome(verdict) -> str:
    return verdict.reason.value if isinstance(verdict, Rejection) else "admitted"


def verify_peer_identity_2890_behavior() -> dict:
    checks = []
    secret = SecretReference(name="lumen-peer-tls")
    replicated = PeerIdentitySpec(
        profile=PeerProfile.PRODUCTION,
        replicas_per_shard=3,
        peer_tls_secret=secret,
    )
    projection = decide_peer_tls_projection(replicated)

    # 1. R1/R2 -- the Secret contract is exactly the three PEM-material keys.
    obs1 = projection.secret_keys if projection is not None else ()
    exp1 = PEER_IDENTITY_2890_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": PEER_IDENTITY_2890_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. R2 -- credentials projected into a member must not be writable there.
    obs2 = projection.mount.access_mode if projection is not None else ""
    exp2 = PEER_IDENTITY_2890_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": PEER_IDENTITY_2890_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. R2 -- the peer TLS material has one named Lumen mount location.
    obs3 = projection.mount.path if projection is not None else ""
    exp3 = PEER_IDENTITY_2890_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": PEER_IDENTITY_2890_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. R2 -- the four bindings are a closed contract, not a partial hint.
    obs4 = tuple(projection.environment) if projection is not None else ()
    exp4 = PEER_IDENTITY_2890_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": PEER_IDENTITY_2890_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. R1/R2 -- an explicit absent reference never manufactures a projection.
    no_secret = PeerIdentitySpec(profile=PeerProfile.DEVELOPMENT, replicas_per_shard=1, peer_tls_secret=None)
    obs5 = decide_peer_tls_projection(no_secret)
    exp5 = PEER_IDENTITY_2890_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": PEER_IDENTITY_2890_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    transport = decide_raft_transport(replicated)

    # 6. R3 -- replication selects mutual TLS rather than a public h2c route.
    obs6 = transport.mode
    exp6 = PEER_IDENTITY_2890_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": PEER_IDENTITY_2890_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. R3 -- its peer protocol is HTTPS.
    obs7 = transport.scheme
    exp7 = PEER_IDENTITY_2890_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": PEER_IDENTITY_2890_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. R3 -- the peer listener is dedicated to port 7374.
    obs8 = transport.port
    exp8 = PEER_IDENTITY_2890_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": PEER_IDENTITY_2890_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. R3/R7 -- one member is explicitly a no-peer topology.
    single_transport = decide_raft_transport(no_secret)
    obs9 = single_transport.mode
    exp9 = PEER_IDENTITY_2890_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": PEER_IDENTITY_2890_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    expectation = peer_identity_expectation(
        "lumen-orders-0.lumen-orders.default.svc.cluster.local", "lumen.axiom.dev"
    )

    # 10. R5 -- verification is bound to the caller-supplied headless DNS name.
    obs10 = expectation.server_dns
    exp10 = PEER_IDENTITY_2890_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": PEER_IDENTITY_2890_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. R5 -- the instance trust domain is preserved in the expectation.
    obs11 = expectation.trust_domain
    exp11 = PEER_IDENTITY_2890_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": PEER_IDENTITY_2890_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. R5 -- peer callers require a client certificate, not server TLS alone.
    obs12 = expectation.client_certificate
    exp12 = PEER_IDENTITY_2890_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": PEER_IDENTITY_2890_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. R7 -- development's exception is only the explicit single-member mode.
    development = decide_profile_peer_tls(PeerProfile.DEVELOPMENT, 1, None)
    obs13 = development.mode if not isinstance(development, Rejection) else _outcome(development)
    exp13 = PEER_IDENTITY_2890_BEHAVIOR_MATRIX[12][1]
    checks.append({"name": PEER_IDENTITY_2890_BEHAVIOR_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    # 14. R4/R7 -- no peer material is required in the explicit single-member state.
    single_status = decide_peer_identity_status(no_secret, PeerMaterialState.ABSENT)
    obs14 = single_status.condition.reason
    exp14 = PEER_IDENTITY_2890_BEHAVIOR_MATRIX[13][1]
    checks.append({"name": PEER_IDENTITY_2890_BEHAVIOR_MATRIX[13][0], "expected": exp14, "observed": obs14, "passed": obs14 == exp14})

    return {
        "case_id": "peer-identity-2890-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) == MINIMUM_CHECKS,
    }
