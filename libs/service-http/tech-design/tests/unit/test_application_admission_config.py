from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_http.application.admission_config import (
    AdmissionConfig,
    controller_policies,
    from_lookup,
    is_enabled,
    policies,
)
from service_http.domain.errors import (
    InvalidValue,
    OrphanedCommonSetting,
)


class TestApplicationAdmissionConfig(unittest.TestCase):
    def test_empty_lookup_defaults(self) -> None:
        data: dict[str, str] = {}
        cfg = from_lookup("LUMEN", data.get)
        self.assertEqual(
            cfg, AdmissionConfig(None, None, None, 60, 1024)
        )
        self.assertFalse(is_enabled(cfg))
        self.assertIsNone(controller_policies(cfg, "read", "write", "admin"))

    def test_populated_lookup_capacities(self) -> None:
        data = {
            "LUMEN_ADMISSION_READ_CAPACITY": "5",
            "LUMEN_ADMISSION_WRITE_CAPACITY": "3",
            "LUMEN_ADMISSION_ADMIN_CAPACITY": "1",
            "LUMEN_ADMISSION_REFILL_SECS": "30",
            "LUMEN_ADMISSION_MAX_KEYS": "16",
        }
        cfg = from_lookup("LUMEN", data.get)
        self.assertEqual(cfg, AdmissionConfig(5, 3, 1, 30, 16))
        self.assertTrue(is_enabled(cfg))

    def test_invalid_value_read_key(self) -> None:
        data = {
            "LUMEN_ADMISSION_READ_CAPACITY": "x",
            "LUMEN_ADMISSION_MAX_KEYS": "y",
        }
        res = from_lookup("LUMEN", data.get)
        self.assertEqual(
            res, InvalidValue("LUMEN_ADMISSION_READ_CAPACITY", "x")
        )

    def test_invalid_value_zero(self) -> None:
        data = {"LUMEN_ADMISSION_READ_CAPACITY": "0"}
        res = from_lookup("LUMEN", data.get)
        self.assertEqual(
            res, InvalidValue("LUMEN_ADMISSION_READ_CAPACITY", "0")
        )

    def test_orphaned_common_setting_refill(self) -> None:
        data = {
            "LUMEN_ADMISSION_REFILL_SECS": "30",
            "LUMEN_ADMISSION_MAX_KEYS": "16",
        }
        res = from_lookup("LUMEN", data.get)
        self.assertEqual(
            res, OrphanedCommonSetting("LUMEN_ADMISSION_REFILL_SECS")
        )

    def test_orphaned_common_setting_max_keys(self) -> None:
        data = {"LUMEN_ADMISSION_MAX_KEYS": "16"}
        res = from_lookup("LUMEN", data.get)
        self.assertEqual(
            res, OrphanedCommonSetting("LUMEN_ADMISSION_MAX_KEYS")
        )

    def test_no_orphan_when_capacity_present(self) -> None:
        data = {
            "LUMEN_ADMISSION_WRITE_CAPACITY": "3",
            "LUMEN_ADMISSION_REFILL_SECS": "30",
        }
        cfg = from_lookup("LUMEN", data.get)
        self.assertIsInstance(cfg, AdmissionConfig)
        self.assertTrue(is_enabled(cfg))

    def test_controller_policies_none_when_disabled(self) -> None:
        cfg = AdmissionConfig(None, None, None, 60, 1024)
        self.assertIsNone(controller_policies(cfg, "read", "write", "admin"))

    def test_policies_write_only(self) -> None:
        data = {"LUMEN_ADMISSION_WRITE_CAPACITY": "3"}
        cfg = from_lookup("LUMEN", data.get)
        self.assertIsInstance(cfg, AdmissionConfig)
        p_dict = policies(cfg, "read", "write", "admin")
        self.assertEqual(len(p_dict), 1)
        self.assertIn("write", p_dict)
        self.assertEqual(p_dict["write"].refill_window_ns, 60_000_000_000)
