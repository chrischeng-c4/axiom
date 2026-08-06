from __future__ import annotations

import importlib.util
import pathlib
import sys
import unittest

SOURCE = pathlib.Path(__file__).parents[2] / "src" / "lumen" / "issuer_ownership.py"
SPEC = importlib.util.spec_from_file_location("lumen_issuer_ownership", SOURCE)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class TestIssuerOwnership(unittest.TestCase):
    def test_complete_matrix_matches_independent_contract(self) -> None:
        expected_allowed = {"namespace", "image", "monitoring"}
        expected_secrets = {"servingTlsSecret", "peerTlsSecret"}
        expected_forbidden = {
            "--issuer",
            "--trust-domain",
            "--ca-pool",
            "LUMEN_ISSUER",
            "LUMEN_TRUST_DOMAIN",
            "LUMEN_CA_POOL",
            "cas-resolver",
            "metadata-token-source",
            "cas",
            "ephemeral",
        }
        self.assertEqual(
            set(MODULE.EXPECTED_CLASSIFICATIONS),
            expected_allowed | expected_secrets | expected_forbidden,
        )
        self.assertEqual(
            {surface for surface in MODULE.EXPECTED_CLASSIFICATIONS
             if MODULE.classify(surface) is MODULE.Classification.OPERATOR_INPUT},
            expected_allowed,
        )
        self.assertEqual(
            {surface for surface in MODULE.EXPECTED_CLASSIFICATIONS
             if MODULE.classify(surface) is MODULE.Classification.EXTERNAL_SECRET},
            expected_secrets,
        )
        self.assertEqual(
            {surface for surface in MODULE.EXPECTED_CLASSIFICATIONS
             if MODULE.classify(surface) is MODULE.Classification.RETIRED_FORBIDDEN},
            expected_forbidden,
        )
        report = MODULE.evaluate()
        self.assertTrue(report.passed)
        self.assertEqual(len(report.checks), len(expected_allowed | expected_secrets | expected_forbidden))

    def test_unknown_surface_is_rejected(self) -> None:
        with self.assertRaises(ValueError):
            MODULE.classify("future-issuer-surface")


if __name__ == "__main__":
    unittest.main()
