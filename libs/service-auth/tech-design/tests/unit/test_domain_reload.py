"""Unit tests for registry reload validation rules."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
if str(SRC_ROOT) not in sys.path:
    sys.path.insert(0, str(SRC_ROOT))

from service_auth.application.reload_registry import validate
from service_auth.domain.claims import TokenClaims
from service_auth.domain.registry import Registry
from service_auth.domain.role import Role


class TestDomainReload(unittest.TestCase):
    def test_five_validation_rules(self) -> None:
        # Rule 1: required_but_empty
        empty_reg = Registry(tokens={}, identities={})
        self.assertEqual(validate(True, empty_reg), "required_but_empty")
        self.assertIsNone(validate(False, empty_reg))

        # Rule 2: empty_key
        empty_key_reg = Registry(
            tokens={"  ": TokenClaims("sub", {"*": Role.READ})},
            identities={},
        )
        self.assertEqual(validate(False, empty_key_reg), "empty_key")

        # Rule 3: empty_subject
        empty_sub_reg = Registry(
            tokens={"sec": TokenClaims("   ", {"*": Role.READ})},
            identities={},
        )
        self.assertEqual(validate(False, empty_sub_reg), "empty_subject")

        # Rule 4: empty_resource
        empty_res_reg = Registry(
            tokens={"sec": TokenClaims("sub", {" ": Role.READ})},
            identities={},
        )
        self.assertEqual(validate(False, empty_res_reg), "empty_resource")

        # Rule 5: identity_key_not_an_email
        bad_identity_reg = Registry(
            tokens={},
            identities={"not_an_email": TokenClaims("sub", {"*": Role.READ})},
        )
        self.assertEqual(validate(False, bad_identity_reg), "identity_key_not_an_email")

        # Valid registry passes all rules
        valid_reg = Registry(
            tokens={"sec": TokenClaims("sub", {"res": Role.READ})},
            identities={"user@example.com": TokenClaims("sub2", {"*": Role.ADMIN})},
        )
        self.assertIsNone(validate(True, valid_reg))

    def test_entry_nesting_beats_rule_ordering(self) -> None:
        registry = Registry(
            tokens={"sec": TokenClaims("   ", {"*": Role.READ})},
            identities={"  ": TokenClaims("sub", {"*": Role.READ})},
        )
        self.assertEqual(validate(False, registry), "empty_subject")


if __name__ == "__main__":
    unittest.main()
