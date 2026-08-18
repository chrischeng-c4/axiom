from __future__ import annotations

import importlib.util
import pathlib
import sys
import unittest

SOURCE = pathlib.Path(__file__).parents[2] / "src" / "lumen" / "trust_anchor_handoff.py"
SPEC = importlib.util.spec_from_file_location("lumen_trust_anchor_handoff", SOURCE)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)

EXPECTED_ALLOWED = {"--ca-file", "PrivateTrust"}
EXPECTED_EXTERNAL = {"servingTlsSecret", "public-ca"}
EXPECTED_FORBIDDEN = {
    "ConfigMap-publisher",
    "status-discovery",
    "pod-kubernetes-writer",
    "trust-bundle-Role",
    "trust-bundle-RoleBinding",
    "automatic-ca-publication",
}


class TestTrustAnchorHandoff(unittest.TestCase):
    def test_exact_closed_matrix(self) -> None:
        expected = EXPECTED_ALLOWED | EXPECTED_EXTERNAL | EXPECTED_FORBIDDEN
        self.assertEqual(set(MODULE.EXPECTED_CLASSIFICATIONS), expected)
        self.assertEqual(
            {surface for surface in expected if MODULE.classify(surface) is MODULE.Classification.CLIENT_INPUT},
            EXPECTED_ALLOWED,
        )
        self.assertEqual(
            {surface for surface in expected if MODULE.classify(surface) is MODULE.Classification.EXTERNAL_HANDOFF},
            EXPECTED_EXTERNAL,
        )
        self.assertEqual(
            {surface for surface in expected if MODULE.classify(surface) is MODULE.Classification.FORBIDDEN_PUBLISHER},
            EXPECTED_FORBIDDEN,
        )

    def test_unknown_surface_fails_closed(self) -> None:
        with self.assertRaises(ValueError):
            MODULE.classify("future-trust-publisher")


if __name__ == "__main__":
    unittest.main()
