from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from cli_std.application.token_resolution import (
    SecretNotFound,
    UndecodableSecret,
    resolve_token,
    uses_cluster,
)
from cli_std.domain.authz import Role


def dummy_json_loader(raw_bytes: bytes) -> object | None:
    if raw_bytes == b"not_json":
        return None
    import json
    return json.loads(raw_bytes.decode("utf-8"))


def dummy_secret_reader(namespace: str, name: str) -> object | None:
    if name == "missing":
        return None
    if name == "bad_b64":
        return {"data": {"token-registry.json": "not_b64!"}}
    if name == "bad_json":
        return {"data": {"token-registry.json": "bm90X2pzb24="}}  # b"not_json"
    if name == "bad_registry":
        return {"data": {"token-registry.json": "eyJ0b2tlbnMiOiAzfQ=="}}  # {"tokens": 3}
    if name == "valid":
        # {"t_docs": {"subject": "s1", "roles": {"docs": "read", "*": "admin"}}, "t_write": {"subject": "s2", "roles": {"*": "write"}}}
        return {
            "data": {
                "token-registry.json": "eyJ0X2RvY3MiOiB7InN1YmplY3QiOiAiczEiLCAicm9sZXMiOiB7ImRvY3MiOiAicmVhZCIsICIqIjogImFkbWluIn19LCAidF93cml0ZSI6IHsic3ViamVjdCI6ICJzMiIsICJyb2xlcyI6IHsiKiI6ICJ3cml0ZSJ9fX0="
            }
        }
    if name == "single_token":
        # {"t_docs": {"subject": "s1", "roles": {"docs": "read", "*": "admin"}}}
        return {
            "data": {
                "token-registry.json": "eyJ0X2RvY3MiOiB7InN1YmplY3QiOiAiczEiLCAicm9sZXMiOiB7ImRvY3MiOiAicmVhZCIsICIqIjogImFkbWluIn19fQ=="
            }
        }
    return None


def exploding_secret_reader(namespace: str, name: str) -> object | None:
    raise AssertionError(
        "read_secret should not be called when explicit token is provided"
    )


class TestApplicationTokenResolution(unittest.TestCase):
    def test_resolve_token_explicit_token_short_circuits(self) -> None:
        res = resolve_token(
            "my_tok",
            "ns",
            "sec",
            Role.ADMIN,
            "res",
            dummy_secret_reader,
            dummy_json_loader,
        )
        self.assertEqual(res, "my_tok")

    def test_resolve_token_exploding_reader_short_circuit(self) -> None:
        res = resolve_token(
            "my_tok",
            "ns",
            "sec",
            Role.ADMIN,
            "res",
            exploding_secret_reader,
            dummy_json_loader,
        )
        self.assertEqual(res, "my_tok")

    def test_resolve_token_missing_namespace_returns_none(self) -> None:
        res = resolve_token(
            None,
            None,
            "sec",
            Role.READ,
            None,
            exploding_secret_reader,
            dummy_json_loader,
        )
        self.assertIsNone(res)

    def test_resolve_token_missing_secret_name_returns_none(self) -> None:
        res = resolve_token(
            None,
            "ns",
            None,
            Role.READ,
            None,
            exploding_secret_reader,
            dummy_json_loader,
        )
        self.assertIsNone(res)

    def test_resolve_token_secret_not_found(self) -> None:
        res = resolve_token(
            None,
            "ns",
            "missing",
            Role.READ,
            None,
            dummy_secret_reader,
            dummy_json_loader,
        )
        self.assertEqual(res, SecretNotFound("ns", "missing"))

    def test_resolve_token_undecodable_base64(self) -> None:
        res = resolve_token(
            None,
            "ns",
            "bad_b64",
            Role.READ,
            None,
            dummy_secret_reader,
            dummy_json_loader,
        )
        self.assertIsInstance(res, UndecodableSecret)

    def test_resolve_token_unparseable_json(self) -> None:
        res = resolve_token(
            None,
            "ns",
            "bad_json",
            Role.READ,
            None,
            dummy_secret_reader,
            dummy_json_loader,
        )
        self.assertIsInstance(res, UndecodableSecret)
        if isinstance(res, UndecodableSecret):
            self.assertEqual(res.reason, "not json")

    def test_resolve_token_malformed_registry(self) -> None:
        res = resolve_token(
            None,
            "ns",
            "bad_registry",
            Role.READ,
            None,
            dummy_secret_reader,
            dummy_json_loader,
        )
        self.assertIsInstance(res, UndecodableSecret)

    def test_resolve_token_resource_specific_grant_preferred(self) -> None:
        res_read_docs = resolve_token(
            None,
            "ns",
            "valid",
            Role.READ,
            "docs",
            dummy_secret_reader,
            dummy_json_loader,
        )
        self.assertEqual(res_read_docs, "t_docs")

    def test_resolve_token_wildcard_fallback_when_resource_absent(self) -> None:
        res_write_other = resolve_token(
            None,
            "ns",
            "valid",
            Role.WRITE,
            "other",
            dummy_secret_reader,
            dummy_json_loader,
        )
        self.assertEqual(res_write_other, "t_docs")

    def test_resolve_token_no_covering_token_returns_none(self) -> None:
        res_write_docs = resolve_token(
            None,
            "ns",
            "single_token",
            Role.WRITE,
            "docs",
            dummy_secret_reader,
            dummy_json_loader,
        )
        self.assertIsNone(res_write_docs)

    def test_resolve_token_admin_covers_read_read_not_write(self) -> None:
        res_read = resolve_token(
            None,
            "ns",
            "valid",
            Role.READ,
            None,
            dummy_secret_reader,
            dummy_json_loader,
        )
        self.assertEqual(res_read, "t_docs")

    def test_uses_cluster_helper(self) -> None:
        self.assertFalse(uses_cluster("tok", "ns", "sec"))
        self.assertTrue(uses_cluster(None, "ns", "sec"))
        self.assertFalse(uses_cluster(None, None, "sec"))
        self.assertFalse(uses_cluster(None, "ns", None))


if __name__ == "__main__":
    unittest.main()
