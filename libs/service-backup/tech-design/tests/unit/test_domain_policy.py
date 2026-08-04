from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_backup.domain.destination import Local
from service_backup.domain.policy import (
    BackupPolicy,
    Retention,
    ScheduledBackupPolicy,
    is_blank_schedule,
    is_expired,
    prunes_by_age,
)


class TestDomainPolicy(unittest.TestCase):
    def test_is_blank_schedule(self) -> None:
        self.assertTrue(is_blank_schedule(""))
        self.assertTrue(is_blank_schedule("  "))
        self.assertTrue(is_blank_schedule("\t\n"))
        self.assertFalse(is_blank_schedule("0 * * * *"))
        self.assertFalse(is_blank_schedule(" 0 * * * * "))

    def test_prunes_by_age(self) -> None:
        self.assertFalse(prunes_by_age(Retention()))
        self.assertFalse(prunes_by_age(Retention(None)))
        self.assertTrue(prunes_by_age(Retention(0)))
        self.assertTrue(prunes_by_age(Retention(3600)))

    def test_is_expired_none_retention(self) -> None:
        self.assertFalse(is_expired(0, 10_000, Retention(None)))

    def test_is_expired_zero_retention(self) -> None:
        self.assertTrue(is_expired(0, 10_000, Retention(0)))
        self.assertTrue(is_expired(9_999, 10_000, Retention(0)))
        self.assertFalse(is_expired(10_000, 10_000, Retention(0)))
        self.assertFalse(is_expired(10_001, 10_000, Retention(0)))

    def test_is_expired_positive_retention(self) -> None:
        self.assertTrue(is_expired(6_399, 10_000, Retention(3600)))
        self.assertFalse(is_expired(6_400, 10_000, Retention(3600)))
        self.assertFalse(is_expired(6_401, 10_000, Retention(3600)))

    def test_is_expired_negative_cutoff(self) -> None:
        self.assertFalse(is_expired(0, 10_000, Retention(20_000)))

    def test_policy_defaults(self) -> None:
        p1 = BackupPolicy(schedule="0 * * * *", destination=Local("/tmp"))
        p2 = BackupPolicy(schedule="0 * * * *", destination=Local("/tmp"))
        self.assertIsNot(p1.retention, p2.retention)
        self.assertEqual(p1.retention, Retention())

    def test_scheduled_backup_policy_fields(self) -> None:
        sp = ScheduledBackupPolicy("0 * * * *", "s3://b/p", 3600)
        self.assertEqual(sp.schedule, "0 * * * *")
        self.assertEqual(sp.destination, "s3://b/p")
        self.assertEqual(sp.retention_secs, 3600)


if __name__ == "__main__":
    unittest.main()
