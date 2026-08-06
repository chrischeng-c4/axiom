from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from cli_std.domain.authz import Role, TokenClaims
from cli_std.domain.registry import (
    MalformedClaims,
    NotAnObject,
    bearer_secrets,
    is_namespaced,
    role_from_name,
)


class TestDomainRegistry(unittest.TestCase):
    def test_bearer_secrets_flat_registry(self) -> None:
        doc = {
            "t1": {"subject": "s1", "roles": {"docs": "read"}},
            "t2": {"subject": "s2", "roles": {"*": "admin"}},
        }
        res = bearer_secrets(doc)
        self.assertIsInstance(res, dict)
        if isinstance(res, dict):
            self.assertEqual(len(res), 2)
            self.assertEqual(res["t1"], TokenClaims("s1", {"docs": Role.READ}))
            self.assertEqual(res["t2"], TokenClaims("s2", {"*": Role.ADMIN}))

    def test_bearer_secrets_namespaced_registry(self) -> None:
        doc = {
            "tokens": {"t1": {"subject": "s1", "roles": {"*": "write"}}},
            "identities": {"id1": {"subject": "user@example.com"}},
        }
        res = bearer_secrets(doc)
        self.assertIsInstance(res, dict)
        if isinstance(res, dict):
            self.assertEqual(len(res), 1)
            self.assertIn("t1", res)
            self.assertNotIn("id1", res)

    def test_bearer_secrets_identities_never_presentable(self) -> None:
        doc = {
            "tokens": {"a": {"subject": "s"}},
            "identities": {"b": {"subject": "e"}},
        }
        res = bearer_secrets(doc)
        self.assertIsInstance(res, dict)
        if isinstance(res, dict):
            self.assertEqual(set(res.keys()), {"a"})

    def test_bearer_secrets_flat_registry_named_tokens(self) -> None:
        doc = {"tokens": {"subject": "s", "roles": {"*": "admin"}}}
        res = bearer_secrets(doc)
        self.assertIsInstance(res, dict)
        if isinstance(res, dict):
            self.assertEqual(
                res, {"tokens": TokenClaims("s", {"*": Role.ADMIN})}
            )

    def test_bearer_secrets_empty_document(self) -> None:
        res = bearer_secrets({})
        self.assertIsInstance(res, dict)
        if isinstance(res, dict):
            self.assertEqual(res, {})

    def test_bearer_secrets_non_object(self) -> None:
        self.assertIsInstance(bearer_secrets(3), NotAnObject)
        self.assertIsInstance(bearer_secrets("x"), NotAnObject)
        self.assertIsInstance(bearer_secrets(None), NotAnObject)
        self.assertIsInstance(bearer_secrets([]), NotAnObject)

    def test_bearer_secrets_namespaced_tokens_non_mapping(self) -> None:
        doc = {"tokens": 3}
        self.assertIsInstance(bearer_secrets(doc), NotAnObject)

    def test_bearer_secrets_claims_missing_subject(self) -> None:
        self.assertIsInstance(bearer_secrets({"t": 3}), MalformedClaims)
        self.assertIsInstance(bearer_secrets({"t": {}}), MalformedClaims)
        self.assertIsInstance(
            bearer_secrets({"t": {"subject": 1}}), MalformedClaims
        )

    def test_bearer_secrets_role_from_string(self) -> None:
        doc = {"t": {"subject": "s", "roles": {"r": "read", "w": "write"}}}
        res = bearer_secrets(doc)
        self.assertIsInstance(res, dict)
        if isinstance(res, dict):
            self.assertEqual(res["t"].roles["r"], Role.READ)
            self.assertEqual(res["t"].roles["w"], Role.WRITE)

    def test_bearer_secrets_role_from_enum_member(self) -> None:
        doc = {"t": {"subject": "s", "roles": {"d": Role.READ}}}
        res = bearer_secrets(doc)
        self.assertIsInstance(res, dict)
        if isinstance(res, dict):
            self.assertEqual(res["t"].roles["d"], Role.READ)

    def test_bearer_secrets_unknown_role_name(self) -> None:
        doc = {"t": {"subject": "s", "roles": {"d": "READ"}}}
        res = bearer_secrets(doc)
        self.assertIsInstance(res, MalformedClaims)

    def test_role_from_name_case_sensitivity(self) -> None:
        self.assertEqual(role_from_name("read"), Role.READ)
        self.assertEqual(role_from_name("write"), Role.WRITE)
        self.assertEqual(role_from_name("admin"), Role.ADMIN)

        self.assertIsNone(role_from_name("READ"))
        self.assertIsNone(role_from_name("Admin"))
        self.assertIsNone(role_from_name(""))
        self.assertIsNone(role_from_name("r"))

    def test_is_namespaced_discriminator(self) -> None:
        self.assertTrue(is_namespaced({}))
        self.assertTrue(is_namespaced({"tokens": {}, "identities": {}}))
        self.assertFalse(is_namespaced({"tokens": {"subject": "s"}}))
        self.assertFalse(is_namespaced({"tk": {"subject": "s"}}))


if __name__ == "__main__":
    unittest.main()
