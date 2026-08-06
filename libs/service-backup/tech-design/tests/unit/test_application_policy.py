from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_backup.application.policy import to_runtime_policy
from service_backup.domain.destination import Local, S3
from service_backup.domain.errors import EmptyDestination, EmptySchedule, UnsupportedScheme
from service_backup.domain.policy import BackupPolicy, Retention, ScheduledBackupPolicy


class TestApplicationPolicy(unittest.TestCase):
    def test_to_runtime_policy_success(self) -> None:
        sp = ScheduledBackupPolicy("0 * * * *", "s3://bucket/prefix", 3600)
        expected = BackupPolicy(
            schedule="0 * * * *",
            destination=S3(bucket="bucket", prefix="prefix"),
            retention=Retention(3600),
        )
        self.assertEqual(to_runtime_policy(sp), expected)

    def test_to_runtime_policy_blank_schedule_first(self) -> None:
        sp1 = ScheduledBackupPolicy("  ", "s3://bucket/prefix", 3600)
        self.assertEqual(to_runtime_policy(sp1), EmptySchedule())

        sp2 = ScheduledBackupPolicy("", "ftp://x", None)
        self.assertEqual(to_runtime_policy(sp2), EmptySchedule())

    def test_to_runtime_policy_unsupported_destination(self) -> None:
        sp = ScheduledBackupPolicy("0 * * * *", "ftp://x", None)
        schemes = ("file://", "s3://", "gs://")
        self.assertEqual(to_runtime_policy(sp), UnsupportedScheme("ftp://x", schemes))

    def test_to_runtime_policy_empty_destination(self) -> None:
        sp = ScheduledBackupPolicy("0 * * * *", "", 1)
        self.assertEqual(to_runtime_policy(sp), EmptyDestination())

    def test_to_runtime_policy_whitespace_and_local(self) -> None:
        sp1 = ScheduledBackupPolicy(" 0 * * * * ", "  s3://b  ", None)
        expected1 = BackupPolicy(
            schedule=" 0 * * * * ",
            destination=S3(bucket="b", prefix=""),
            retention=Retention(None),
        )
        self.assertEqual(to_runtime_policy(sp1), expected1)

        sp2 = ScheduledBackupPolicy("s", "file:///d", 0)
        expected2 = BackupPolicy(
            schedule="s",
            destination=Local(path="/d"),
            retention=Retention(0),
        )
        self.assertEqual(to_runtime_policy(sp2), expected2)


if __name__ == "__main__":
    unittest.main()
