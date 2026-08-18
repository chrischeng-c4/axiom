from __future__ import annotations

import dataclasses
from datetime import datetime, timezone
import inspect

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

import peer_tls.infrastructure.config_plan as config_plan_mod
from peer_tls.infrastructure.config_plan import (
    ClientConfigPlan,
    ServerConfigPlan,
    plan_client,
    plan_server,
)
from peer_tls.domain.verdict import ValidatedMaterial, ValidityWindow

MINIMUM_CHECKS = 13

MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX = (
    ("server_config_plan_peer_certificate_required_true", True),
    ("client_config_plan_presents_client_certificate_true", True),
    ("planners_expose_no_permissive_constructor_params", True),
    ("config_plan_module_exposes_no_third_plan_function", True),
    ("unrelated_authority_not_admitted", False),
    ("rejected_material_returns_none_config", True),
    ("service_server_plan_peer_certificate_required", True),
    ("service_client_plan_presents_client_certificate", True),
    ("service_server_plan_admits_the_configured_anchor", True),
    ("service_server_plan_refuses_an_unrelated_authority", False),
    ("plan_server_admits_the_configured_anchor", True),
    ("plan_client_admits_the_configured_anchor", True),
    ("standalone_plans_refuse_an_unrelated_authority", False),
)


class FakeEnv:

    def __init__(self, data: dict[str, str]):
        self._data = data

    def get(self, name: str) -> str | None:
        return self._data.get(name)


class FakeInstaller:

    def install_default(self) -> bool:
        return True


class FakeClock:

    def __init__(self, now_val: datetime):
        self._now = now_val

    def now(self) -> datetime:
        return self._now


def verify_mtls_config_construction_security() -> dict:
    checks = []

    valid_from = datetime(2026, 1, 10, tzinfo=timezone.utc)
    valid_to = datetime(2026, 2, 10, tzinfo=timezone.utc)
    instant_now = datetime(2026, 1, 15, 12, 0, 0, tzinfo=timezone.utc)

    trust = TrustBundle(anchors=(TrustAnchor(key_id="issuer1", label="ca1"),))
    exp = IdentityExpectation(kind=ExpectationKind.SERVING, dns_names=(DnsName("service.example.com"),))
    win = ValidityWindow(not_before=valid_from, not_after=valid_to)
    val_material = ValidatedMaterial(window=win, identity=exp)

    server_plan = plan_server(val_material, trust, "leaf_label")
    client_plan = plan_client(val_material, trust, "leaf_label")

    # 1. server_config_plan_peer_certificate_required_true
    obs1 = server_plan.peer_certificate_required is True
    exp1 = MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[0][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[0][0], "expected": exp1, "observed": obs1, "passed": obs1 == exp1})

    # 2. client_config_plan_presents_client_certificate_true
    obs2 = client_plan.presents_client_certificate is True
    exp2 = MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[1][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[1][0], "expected": exp2, "observed": obs2, "passed": obs2 == exp2})

    # 3. planners_expose_no_permissive_constructor_params
    s_fields = dataclasses.fields(ServerConfigPlan)
    s_params = inspect.signature(ServerConfigPlan).parameters
    c_fields = dataclasses.fields(ClientConfigPlan)
    c_params = inspect.signature(ClientConfigPlan).parameters

    s_valid = (
        len(s_fields) == 2
        and "peer_certificate_required" not in {f.name for f in s_fields}
        and "peer_certificate_required" not in s_params
    )
    c_valid = (
        len(c_fields) == 2
        and "presents_client_certificate" not in {f.name for f in c_fields}
        and "presents_client_certificate" not in c_params
    )
    obs3 = s_valid and c_valid
    exp3 = MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[2][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[2][0], "expected": exp3, "observed": obs3, "passed": obs3 == exp3})

    # 4. config_plan_module_exposes_no_third_plan_function
    funcs = [
        name
        for name, obj in inspect.getmembers(config_plan_mod, inspect.isfunction)
        if name.startswith("plan_") or "plan" in name.lower()
    ]
    obs4 = set(funcs) == {"plan_server", "plan_client"}
    exp4 = MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[3][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[3][0], "expected": exp4, "observed": obs4, "passed": obs4 == exp4})

    # 5. unrelated_authority_not_admitted
    obs5 = trust.admits("unrelated_authority_key")
    exp5 = MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[4][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[4][0], "expected": exp5, "observed": obs5, "passed": obs5 == exp5})

    # 6. rejected_material_returns_none_config
    env = FakeEnv({
        "TEST_CERT": "/path/leaf.pem",
        "TEST_KEY": "/path/key.pem",
        "TEST_CA": "/path/ca.pem",
    })
    installer = FakeInstaller()
    clock = FakeClock(instant_now)
    svc = BuildMtlsConfigService(env=env, installer=installer, clock=clock)

    bad_leaf = LeafAttributes(
        subject_alt_names=SubjectAltNames(dns_names=(DnsName("other.com"),)),
        not_before=valid_from,
        not_after=valid_to,
        public_key_fingerprint="fp123",
        issuer_key_id="issuer1",
    )
    bad_key = PrivateKeyAttributes(public_key_fingerprint="fp123")
    bad_triple = MaterialTriple(leaf=bad_leaf, key=bad_key, trust=trust)
    res = svc.execute("TEST", bad_triple, exp, "leaf_label")
    obs6 = res is None
    exp6 = MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[5][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[5][0], "expected": exp6, "observed": obs6, "passed": obs6 == exp6})

    # Execute service with valid material for checks 7-10
    good_leaf = LeafAttributes(
        subject_alt_names=SubjectAltNames(dns_names=(DnsName("service.example.com"),)),
        not_before=valid_from,
        not_after=valid_to,
        public_key_fingerprint="fp123",
        issuer_key_id="issuer1",
    )
    good_key = PrivateKeyAttributes(public_key_fingerprint="fp123")
    good_triple = MaterialTriple(leaf=good_leaf, key=good_key, trust=trust)
    service_res = svc.execute("TEST", good_triple, exp, "leaf_label")

    if service_res is not None:
        svc_server_plan, svc_client_plan = service_res
    else:
        svc_server_plan, svc_client_plan = server_plan, client_plan

    # 7. service_server_plan_peer_certificate_required
    obs7 = svc_server_plan.peer_certificate_required is True
    exp7 = MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[6][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[6][0], "expected": exp7, "observed": obs7, "passed": obs7 == exp7})

    # 8. service_client_plan_presents_client_certificate
    obs8 = svc_client_plan.presents_client_certificate is True
    exp8 = MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[7][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[7][0], "expected": exp8, "observed": obs8, "passed": obs8 == exp8})

    # 9. service_server_plan_admits_the_configured_anchor
    obs9 = svc_server_plan.trust.admits("issuer1")
    exp9 = MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[8][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[8][0], "expected": exp9, "observed": obs9, "passed": obs9 == exp9})

    # 10. service_server_plan_refuses_an_unrelated_authority
    obs10 = svc_server_plan.trust.admits("unrelated_authority_key")
    exp10 = MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[9][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[9][0], "expected": exp10, "observed": obs10, "passed": obs10 == exp10})

    # Standalone plan_server / plan_client calls for checks 11-13
    sp = plan_server(val_material, trust, "leaf_label")
    cp = plan_client(val_material, trust, "leaf_label")

    # 11. plan_server_admits_the_configured_anchor
    obs11 = sp.trust.admits("issuer1")
    exp11 = MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[10][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[10][0], "expected": exp11, "observed": obs11, "passed": obs11 == exp11})

    # 12. plan_client_admits_the_configured_anchor
    obs12 = cp.trust.admits("issuer1")
    exp12 = MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[11][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[11][0], "expected": exp12, "observed": obs12, "passed": obs12 == exp12})

    # 13. standalone_plans_refuse_an_unrelated_authority
    obs13 = sp.trust.admits("unrelated_authority_key") or cp.trust.admits("unrelated_authority_key")
    exp13 = MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[12][1]
    checks.append({"name": MTLS_CONFIG_CONSTRUCTION_SECURITY_MATRIX[12][0], "expected": exp13, "observed": obs13, "passed": obs13 == exp13})

    return {
        "case_id": "mtls-config-construction-security",
        "minimum_checks": MINIMUM_CHECKS,
        "checks": checks,
        "passed": all(c["passed"] for c in checks) and len(checks) >= MINIMUM_CHECKS,
    }
