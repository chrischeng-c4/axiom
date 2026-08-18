from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from cli_std.infrastructure.secret_data import (
    TOKEN_REGISTRY_SECRET_KEY,
    MissingDataKey,
    NotBase64,
    cr_tokens_secret,
    secret_data_bytes,
)


class TestInfrastructureSecretData(unittest.TestCase):
    def test_secret_data_bytes_correct_decode(self) -> None:
        secret_json = {"data": {TOKEN_REGISTRY_SECRET_KEY: "aGVsbG8="}}
        res = secret_data_bytes(secret_json, TOKEN_REGISTRY_SECRET_KEY)
        self.assertEqual(res, b"hello")

    def test_secret_data_bytes_missing_data_key(self) -> None:
        self.assertIsInstance(
            secret_data_bytes({}, TOKEN_REGISTRY_SECRET_KEY), MissingDataKey
        )
        self.assertIsInstance(
            secret_data_bytes({"data": {}}, TOKEN_REGISTRY_SECRET_KEY),
            MissingDataKey,
        )

    def test_secret_data_bytes_missing_specific_key(self) -> None:
        secret_json = {"data": {"other_key": "aGVsbG8="}}
        self.assertIsInstance(
            secret_data_bytes(secret_json, TOKEN_REGISTRY_SECRET_KEY),
            MissingDataKey,
        )

    def test_secret_data_bytes_non_string_value(self) -> None:
        secret_json = {"data": {TOKEN_REGISTRY_SECRET_KEY: 123}}
        self.assertIsInstance(
            secret_data_bytes(secret_json, TOKEN_REGISTRY_SECRET_KEY),
            MissingDataKey,
        )

    def test_secret_data_bytes_refuses_invalid_base64_characters(self) -> None:
        secret_json = {"data": {TOKEN_REGISTRY_SECRET_KEY: "aGVsbG8=!!"}}
        res = secret_data_bytes(secret_json, TOKEN_REGISTRY_SECRET_KEY)
        self.assertIsInstance(res, NotBase64)
        if isinstance(res, NotBase64):
            self.assertEqual(res.key, TOKEN_REGISTRY_SECRET_KEY)

    def test_secret_data_bytes_non_mapping_input(self) -> None:
        self.assertIsInstance(
            secret_data_bytes(3, TOKEN_REGISTRY_SECRET_KEY), MissingDataKey
        )
        self.assertIsInstance(
            secret_data_bytes(None, TOKEN_REGISTRY_SECRET_KEY), MissingDataKey
        )

    def test_cr_tokens_secret_valid_spec(self) -> None:
        cr_json = {"spec": {"tokensSecret": "my-secret-name"}}
        self.assertEqual(cr_tokens_secret(cr_json), "my-secret-name")

    def test_cr_tokens_secret_invalid_and_missing_spec(self) -> None:
        self.assertIsNone(cr_tokens_secret({}))
        self.assertIsNone(cr_tokens_secret({"spec": {}}))
        self.assertIsNone(cr_tokens_secret({"spec": 3}))
        self.assertIsNone(cr_tokens_secret({"spec": {"tokensSecret": 3}}))
        self.assertIsNone(cr_tokens_secret(None))
        self.assertIsNone(cr_tokens_secret(3))


if __name__ == "__main__":
    unittest.main()
