"""Unit tests for domain role map, claims, and registry parsing/merging."""

from __future__ import annotations

import sys
import unittest
from pathlib import Path

SRC_ROOT = Path(__file__).resolve().parents[2] / "src"
if str(SRC_ROOT) not in sys.path:
    sys.path.insert(0, str(SRC_ROOT))

from service_auth.domain.claims import WILDCARD_RESOURCE, TokenClaims, resolve_role
from service_auth.domain.registry import (
    IDENTITIES_SECTION,
    TOKENS_SECTION,
    Registry,
    RegistryError,
    lookup_identity,
    lookup_secret,
    parse,
    reserved_subject_violation,
    try_merge,
)
from service_auth.domain.role import ROLE_ORDER, Role, covers, rank


class TestDomainRoleMap(unittest.TestCase):
    def test_role_ordering_and_covers(self) -> None:
        self.assertEqual(rank(Role.READ), 0)
        self.assertEqual(rank(Role.WRITE), 1)
        self.assertEqual(rank(Role.ADMIN), 2)
        self.assertEqual(ROLE_ORDER, (Role.READ, Role.WRITE, Role.ADMIN))

        self.assertTrue(covers(Role.READ, Role.READ))
        self.assertFalse(covers(Role.READ, Role.WRITE))
        self.assertFalse(covers(Role.READ, Role.ADMIN))

        self.assertTrue(covers(Role.WRITE, Role.READ))
        self.assertTrue(covers(Role.WRITE, Role.WRITE))
        self.assertFalse(covers(Role.WRITE, Role.ADMIN))

        self.assertTrue(covers(Role.ADMIN, Role.READ))
        self.assertTrue(covers(Role.ADMIN, Role.WRITE))
        self.assertTrue(covers(Role.ADMIN, Role.ADMIN))

    def test_claims_resolution_exact_vs_wildcard(self) -> None:
        claims = TokenClaims(
            subject="svc",
            roles={"res1": Role.WRITE, WILDCARD_RESOURCE: Role.ADMIN},
        )
        self.assertEqual(resolve_role(claims, "res1"), Role.WRITE)
        self.assertEqual(resolve_role(claims, "other"), Role.ADMIN)

        no_wildcard = TokenClaims(subject="svc2", roles={"res1": Role.READ})
        self.assertIsNone(resolve_role(no_wildcard, "other"))

    def test_parse_flat_document(self) -> None:
        doc = {
            "sec1": {"subject": "sub1", "roles": {"*": "read"}},
            "sec2": {"subject": "sub2", "roles": {"res": "write"}},
        }
        reg = parse(doc)
        self.assertEqual(reg.len(), 2)
        self.assertEqual(len(reg.tokens), 2)
        self.assertEqual(len(reg.identities), 0)

        secret_claims = lookup_secret(reg, "sec1")
        self.assertIsNotNone(secret_claims)
        assert secret_claims is not None
        self.assertEqual(secret_claims.subject, "sub1")

    def test_parse_namespaced_document(self) -> None:
        doc = {
            "tokens": {"sec1": {"subject": "sub1", "roles": {"*": "read"}}},
            "identities": {"user@example.com": {"subject": "user1", "roles": {"*": "admin"}}},
        }
        reg = parse(doc)
        self.assertEqual(reg.len(), 2)
        self.assertEqual(len(reg.tokens), 1)
        self.assertEqual(len(reg.identities), 1)

        secret_claims = lookup_secret(reg, "sec1")
        self.assertIsNotNone(secret_claims)
        assert secret_claims is not None
        self.assertEqual(secret_claims.subject, "sub1")

        id_claims = lookup_identity(reg, "user@example.com")
        self.assertIsNotNone(id_claims)
        assert id_claims is not None
        self.assertEqual(id_claims.subject, "user1")

    def test_parse_flat_document_with_key_spelled_tokens(self) -> None:
        doc = {
            "tokens": {"subject": "sub_flat", "roles": {"*": "read"}},
        }
        reg = parse(doc)
        self.assertEqual(len(reg.tokens), 1)
        self.assertEqual(len(reg.identities), 0)

        secret_claims = lookup_secret(reg, "tokens")
        self.assertIsNotNone(secret_claims)
        assert secret_claims is not None
        self.assertEqual(secret_claims.subject, "sub_flat")

    def test_try_merge_collisions(self) -> None:
        reg1 = Registry(
            tokens={"sec1": TokenClaims("s1", {"*": Role.READ})},
            identities={"u1@ex.com": TokenClaims("u1", {"*": Role.READ})},
        )
        reg2 = Registry(
            tokens={"sec2": TokenClaims("s2", {"*": Role.WRITE})},
            identities={"u1@ex.com": TokenClaims("u1_dup", {"*": Role.WRITE})},
        )
        with self.assertRaises(RegistryError) as ctx:
            try_merge(reg1, reg2)
        self.assertEqual(ctx.exception.reason, "duplicate_registry_key")

    def test_reserved_subject_violation(self) -> None:
        reg = Registry(
            tokens={
                "b_sec": TokenClaims("reserved_sub", {}),
                "a_sec": TokenClaims("reserved_sub", {}),
            },
            identities={"u1@ex.com": TokenClaims("reserved_sub", {})},
        )
        violation = reserved_subject_violation(reg, ["reserved_sub"])
        self.assertIsNotNone(violation)
        assert violation is not None
        section, key, subject = violation
        self.assertEqual(section, TOKENS_SECTION)
        self.assertEqual(key, "a_sec")
        self.assertEqual(subject, "reserved_sub")


if __name__ == "__main__":
    unittest.main()
