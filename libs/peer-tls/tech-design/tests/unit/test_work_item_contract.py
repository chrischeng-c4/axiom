"""Unit tests verifying work-item contract declarations."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
sys.path.insert(0, str(SRC_ROOT))

from peer_tls.work_items.author_full_typed_python_td_ddd_and_fill_the_python_ec import (  # noqa: E402
    application_modules,
    design_contract,
    domain_modules,
    infrastructure_modules,
)


class TestWorkItemContract(unittest.TestCase):
    def test_design_contract_is_non_empty(self) -> None:
        contract = design_contract()
        self.assertIsInstance(contract, str)
        self.assertTrue(len(contract) > 0)

    def test_layer_module_tuples_match_frozen_tree(self) -> None:
        self.assertEqual(
            domain_modules(),
            (
                "peer_tls.domain.identity",
                "peer_tls.domain.material",
                "peer_tls.domain.verdict",
                "peer_tls.domain.validation",
                "peer_tls.domain.rotation",
            ),
        )
        self.assertEqual(
            application_modules(),
            (
                "peer_tls.application.validate_material",
                "peer_tls.application.build_mtls_config",
                "peer_tls.application.rotate_material",
            ),
        )
        self.assertEqual(
            infrastructure_modules(),
            (
                "peer_tls.infrastructure.ports",
                "peer_tls.infrastructure.env_resolver",
                "peer_tls.infrastructure.config_plan",
            ),
        )


if __name__ == "__main__":
    unittest.main()
