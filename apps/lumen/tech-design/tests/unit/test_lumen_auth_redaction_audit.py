"""Executable TD assertions for WI #2879's bounded shell contract."""

from __future__ import annotations

import importlib.util
from pathlib import Path
from types import ModuleType
import unittest


TD_MODULE = (
    Path(__file__).resolve().parents[2]
    / "src/lumen/work_items/wi_12_18_lumen_auth_phase_2_prove_two_hop_ksa_rbac_authorization_on.py"
)


def load_td_module() -> ModuleType:
    spec = importlib.util.spec_from_file_location("lumen_auth_wi_2879_td", TD_MODULE)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class LumenAuthRedactionAuditTdTest(unittest.TestCase):
    def test_contract_is_frozen(self) -> None:
        module = load_td_module()

        self.assertEqual(
            module.design_contract(), "lumen_auth_redaction_audit_and_destroy"
        )
        self.assertEqual(
            module.redaction_audit_contract(),
            {
                "target": "acceptance/gcp/scripts/verify-lumen-auth.sh",
                "neither": "skip callback and preserve ordinary direct GKE execution",
                "auditor_only": "fail",
                "audit_path_only": "fail",
                "both": "invoke lumen_auth_redaction_audit_and_destroy",
                "required_expansions": "${LUMEN_AUTH_REDACTION_AUDITOR:?required} ${LUMEN_AUTH_REDACTION_AUDIT_PATH:?required}",
                "order": "write lumen-auth-acceptance.json -> auditor -> rm -rf $SECRET_DIR -> SECRET_DIR=\"\" -> final success echo",
                "verification": "bash -n acceptance/gcp/scripts/verify-lumen-auth.sh; python3 -m unittest discover -s apps/lumen/external-contracts/tests/unit",
                "ec_verify": "aw ec verify --project lumen --stage cb --wi 2879",
            },
        )
