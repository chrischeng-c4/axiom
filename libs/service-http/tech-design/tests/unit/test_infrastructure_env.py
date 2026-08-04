from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

import service_http.infrastructure.env as env
from service_http.infrastructure.env import (
    READ_CAPACITY_SUFFIX,
    all_keys,
    capacity_keys,
    common_keys,
    env_key,
)


class TestInfrastructureEnv(unittest.TestCase):
    def test_env_key(self) -> None:
        self.assertEqual(
            env_key("LUMEN", READ_CAPACITY_SUFFIX),
            "LUMEN_ADMISSION_READ_CAPACITY",
        )

    def test_capacity_keys_order(self) -> None:
        self.assertEqual(
            capacity_keys("K"),
            (
                "K_ADMISSION_READ_CAPACITY",
                "K_ADMISSION_WRITE_CAPACITY",
                "K_ADMISSION_ADMIN_CAPACITY",
            ),
        )

    def test_common_keys_order(self) -> None:
        self.assertEqual(
            common_keys("K"),
            ("K_ADMISSION_REFILL_SECS", "K_ADMISSION_MAX_KEYS"),
        )

    def test_all_keys_length_and_distinctness(self) -> None:
        keys = all_keys("K")
        expected = capacity_keys("K") + common_keys("K")
        self.assertEqual(keys, expected)
        self.assertEqual(len(keys), 5)
        self.assertEqual(len(set(keys)), 5)

    def test_parse_positive_namespace_import(self) -> None:
        self.assertIsNone(env.parse_positive("0"))
