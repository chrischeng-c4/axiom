from __future__ import annotations

from datetime import datetime, timezone

from peer_tls.application.build_mtls_config import BuildMtlsConfigService
from peer_tls.domain.identity import DnsName, ExpectationKind, IdentityExpectation
from peer_tls.domain.material import (
    LeafAttributes,
    MaterialTriple,
    PrivateKeyAttributes,
    SubjectAltNames,
    TrustAnchor,
    TrustBundle,
)
from peer_tls.infrastructure.env_resolver import EnvPrefixError, resolve_locations

MINIMUM_CHECKS = 12

MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX = (
    ("resolve_all_variables_set", True),
    ("resolve_none_set", True),
    ("resolve_partial_missing_cert_raises_error", True),
    ("resolve_partial_missing_key_raises_error", True),
    ("resolve_partial_missing_ca_raises_error", True),
    ("installer_called_once_on_first_execute", 1),
    ("installer_called_again_on_second_execute", 2),
    ("order_ab_resolves_the_declared_paths", ("/a/cert", "/a/key", "/a/ca")),
    ("order_ba_resolves_the_same_declared_paths", ("/a/cert", "/a/key", "/a/ca")),
    ("installer_not_called_when_prefix_unset", 0),
    ("installer_not_called_when_prefix_partial", 0),
    ("installer_called_once_when_material_refused", 1),
)


class FakeEnv:

    def __init__(self, data: dict[str, str]):
        self._data = data

    def get(self, name: str) -> str | None:
        return self._data.get(name)


class FakeInstaller:

    def __init__(self):
        self.calls = 0

    def install_default(self) -> bool:
        self.calls += 1
        return True


class FakeClock:

    def __init__(self, now_val: datetime):
        self._now = now_val

    def now(self) -> datetime:
        return self._now


def verify_mtls_config_construction_behavior() -> dict:
    checks = []

    # 1. resolve_all_variables_set
    env_all = FakeEnv({
        "TEST_CERT": "/path/leaf.pem",
        "TEST_KEY": "/path/key.pem",
        "TEST_CA": "/path/ca.pem",
    })
    locs1 = resolve_locations(env_all, "TEST")
    obs1 = (
        locs1 is not None
        and locs1.leaf == "/path/leaf.pem"
        and locs1.key == "/path/key.pem"
        and locs1.trust == "/path/ca.pem"
    )
    exp1 = MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[0][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. resolve_none_set
    env_none = FakeEnv({})
    locs2 = resolve_locations(env_none, "TEST")
    obs2 = locs2 is None
    exp2 = MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[1][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. resolve_partial_missing_cert_raises_error
    env_no_cert = FakeEnv({"TEST_KEY": "/path/key.pem", "TEST_CA": "/path/ca.pem"})
    try:
        resolve_locations(env_no_cert, "TEST")
        obs3 = False
    except EnvPrefixError:
        obs3 = True
    exp3 = MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[2][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. resolve_partial_missing_key_raises_error
    env_no_key = FakeEnv({"TEST_CERT": "/path/leaf.pem", "TEST_CA": "/path/ca.pem"})
    try:
        resolve_locations(env_no_key, "TEST")
        obs4 = False
    except EnvPrefixError:
        obs4 = True
    exp4 = MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[3][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. resolve_partial_missing_ca_raises_error
    env_no_ca = FakeEnv({"TEST_CERT": "/path/leaf.pem", "TEST_KEY": "/path/key.pem"})
    try:
        resolve_locations(env_no_ca, "TEST")
        obs5 = False
    except EnvPrefixError:
        obs5 = True
    exp5 = MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[4][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # Setup domain objects
    now = datetime(2026, 1, 15, 12, 0, 0, tzinfo=timezone.utc)
    clock = FakeClock(now)

    good_leaf = LeafAttributes(
        subject_alt_names=SubjectAltNames(dns_names=(DnsName("service.example.com"),)),
        not_before=datetime(2026, 1, 10, tzinfo=timezone.utc),
        not_after=datetime(2026, 2, 10, tzinfo=timezone.utc),
        public_key_fingerprint="fp123",
        issuer_key_id="ca1",
    )
    good_key = PrivateKeyAttributes(public_key_fingerprint="fp123")
    trust = TrustBundle(anchors=(TrustAnchor(key_id="ca1", label="ca1"),))
    triple = MaterialTriple(leaf=good_leaf, key=good_key, trust=trust)
    expectation = IdentityExpectation(kind=ExpectationKind.SERVING, dns_names=(DnsName("service.example.com"),))

    # 6. installer_called_once_on_first_execute (fresh installer)
    inst6 = FakeInstaller()
    svc6 = BuildMtlsConfigService(env=env_all, installer=inst6, clock=clock)
    res1 = svc6.execute("TEST", triple, expectation, "leaf_label")
    obs6 = inst6.calls if res1 is not None else -1
    exp6 = MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[5][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # 7. installer_called_again_on_second_execute (reuse svc6/inst6 instance)
    res2 = svc6.execute("TEST", triple, expectation, "leaf_label")
    obs7 = inst6.calls if res2 is not None else -1
    exp7 = MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[6][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8 & 9. Permuted initialization orders
    env_multi = FakeEnv({
        "SVC_A_CERT": "/a/cert", "SVC_A_KEY": "/a/key", "SVC_A_CA": "/a/ca",
        "SVC_B_CERT": "/b/cert", "SVC_B_KEY": "/b/key", "SVC_B_CA": "/b/ca",
    })
    ab_a = resolve_locations(env_multi, "SVC_A")
    ab_b = resolve_locations(env_multi, "SVC_B")

    ba_b = resolve_locations(env_multi, "SVC_B")
    ba_a = resolve_locations(env_multi, "SVC_A")

    obs8 = (ab_a.leaf, ab_a.key, ab_a.trust) if ab_a is not None else ()
    exp8 = MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[7][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    obs9 = (ba_a.leaf, ba_a.key, ba_a.trust) if ba_a is not None else ()
    exp9 = MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[8][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. installer_not_called_when_prefix_unset (fresh installer)
    inst10 = FakeInstaller()
    svc10 = BuildMtlsConfigService(env=env_none, installer=inst10, clock=clock)
    res_unset = svc10.execute("TEST", triple, expectation, "leaf_label")
    obs10 = inst10.calls if res_unset is None else -1
    exp10 = MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[9][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # 11. installer_not_called_when_prefix_partial (fresh installer)
    inst11 = FakeInstaller()
    svc11 = BuildMtlsConfigService(env=env_no_cert, installer=inst11, clock=clock)
    try:
        svc11.execute("TEST", triple, expectation, "leaf_label")
    except EnvPrefixError:
        pass
    obs11 = inst11.calls
    exp11 = MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[10][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. installer_called_once_when_material_refused (fresh installer)
    inst12 = FakeInstaller()
    svc12 = BuildMtlsConfigService(env=env_all, installer=inst12, clock=clock)
    bad_key = PrivateKeyAttributes(public_key_fingerprint="fp999")
    bad_triple = MaterialTriple(leaf=good_leaf, key=bad_key, trust=trust)
    res_refused = svc12.execute("TEST", bad_triple, expectation, "leaf_label")
    obs12 = inst12.calls if res_refused is None else -1
    exp12 = MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[11][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_BEHAVIOR_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    return {
        "case_id": "mtls-config-construction-behavior",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
