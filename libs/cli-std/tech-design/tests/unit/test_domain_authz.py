from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from cli_std.domain.authz import Role, TokenClaims, select_token


class TestDomainAuthz(unittest.TestCase):
    def test_role_covers_hierarchy(self) -> None:
        self.assertTrue(Role.ADMIN.covers(Role.READ))
        self.assertTrue(Role.READ.covers(Role.READ))
        self.assertFalse(Role.READ.covers(Role.WRITE))

    def test_select_token_specific_insufficient_no_wildcard_fallback(self) -> None:
        registry = {
            "tk": TokenClaims("s", {"docs": Role.READ, "*": Role.ADMIN})
        }
        res = select_token(registry, Role.WRITE, "docs")
        self.assertIsNone(res)

    def test_select_token_absent_resource_falls_through_to_wildcard(self) -> None:
        registry = {"tk": TokenClaims("s", {"*": Role.ADMIN})}
        res = select_token(registry, Role.WRITE, "docs")
        self.assertEqual(res, "tk")

    def test_select_token_none_resource_wildcard_only(self) -> None:
        registry_named_only = {
            "tk": TokenClaims("s", {"docs": Role.ADMIN})
        }
        res = select_token(registry_named_only, Role.WRITE, None)
        self.assertIsNone(res)

    def test_select_token_first_qualifying_token(self) -> None:
        registry = {
            "t1": TokenClaims("s1", {"*": Role.WRITE}),
            "t2": TokenClaims("s2", {"*": Role.ADMIN}),
        }
        res = select_token(registry, Role.READ, None)
        self.assertEqual(res, "t1")

    def test_token_claims_dataclass_frozen(self) -> None:
        tc = TokenClaims("sub", {"*": Role.READ})
        self.assertEqual(tc.subject, "sub")
        self.assertEqual(tc.roles["*"], Role.READ)

    def test_select_token_empty_registry(self) -> None:
        self.assertIsNone(select_token({}, Role.READ, "res"))

    def test_role_covers_same_level(self) -> None:
        self.assertTrue(Role.WRITE.covers(Role.WRITE))
        self.assertTrue(Role.ADMIN.covers(Role.ADMIN))


if __name__ == "__main__":
    unittest.main()
