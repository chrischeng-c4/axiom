from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_backup.application.runner import (
    BackupObject,
    BackupRunResult,
    object_key,
    plan_backup_run,
    plan_prune,
    run_result_to_json,
)
from service_backup.application.sink import SinkKind
from service_backup.domain.policy import Retention


class TestApplicationRunner(unittest.TestCase):
    def test_object_key(self) -> None:
        self.assertEqual(object_key(SinkKind.LOCAL, "backup", 42), "backup-42.json")
        self.assertEqual(object_key(SinkKind.LOCAL, "snap", 0), "snap-0.json")
        self.assertEqual(object_key(SinkKind.LOCAL, "", 42), "-42.json")
        self.assertEqual(object_key(SinkKind.S3, "", 42), "backup-42.json")
        self.assertEqual(object_key(SinkKind.S3, "/nested/prefix/", 42), "nested/prefix/backup-42.json")
        self.assertEqual(object_key(SinkKind.GCS, "backup", 7), "backup/backup-7.json")
        self.assertEqual(object_key(SinkKind.GCS, "p", 1_700_000_000), "p/backup-1700000000.json")

    def test_plan_prune(self) -> None:
        self.assertEqual(plan_prune(("backup-1.json",), "", 10_000, Retention()), ())
        self.assertEqual(
            plan_prune(("backup-1.json", "backup-9999.json", "backup-10000.json"), "", 10_000, Retention(0)),
            ("backup-1.json", "backup-9999.json"),
        )
        self.assertEqual(
            plan_prune(("p/backup-1.json", "backup-1.json", "p/junk.json"), "p", 10_000, Retention(0)),
            ("p/backup-1.json",),
        )
        self.assertEqual(
            plan_prune(("p/backup-1.json",), "/p/", 10_000, Retention(0)),
            ("p/backup-1.json",),
        )
        self.assertEqual(
            plan_prune(("backup-6399.json", "backup-6400.json"), "", 10_000, Retention(3600)),
            ("backup-6399.json",),
        )
        self.assertEqual(
            plan_prune(("README.md", "backup-1.json"), "", 10_000, Retention(0)),
            ("backup-1.json",),
        )
        self.assertEqual(plan_prune((), "", 10_000, Retention(0)), ())

    def test_plan_prune_input_order_preserved(self) -> None:
        keys = ("backup-3.json", "backup-1.json", "backup-2.json")
        self.assertEqual(plan_prune(keys, "", 10_000, Retention(0)), keys)

    def test_plan_backup_run_fresh_object_survives(self) -> None:
        r1 = plan_backup_run("s3://b", "", SinkKind.S3, 128, 10_000, Retention(0), ())
        self.assertEqual(r1.object.key, "backup-10000.json")
        self.assertEqual(r1.object.bytes, 128)
        self.assertEqual(r1.object.unix_seconds, 10_000)
        self.assertEqual(r1.pruned, 0)

        r2 = plan_backup_run("s3://b", "", SinkKind.S3, 128, 10_000, Retention(0), ("backup-1.json",))
        self.assertEqual(r2.pruned, 1)

    def test_plan_backup_run_none_retention(self) -> None:
        r = plan_backup_run("s3://b", "", SinkKind.S3, 128, 10_000, Retention(), ("backup-1.json",))
        self.assertEqual(r.object.key, "backup-10000.json")
        self.assertEqual(r.pruned, 0)

    def test_plan_backup_run_other_cases(self) -> None:
        r_local = plan_backup_run("local:/p", "backup", SinkKind.LOCAL, 0, 42, Retention(), ())
        self.assertEqual(r_local.object.key, "backup-42.json")
        self.assertEqual(r_local.object.bytes, 0)
        self.assertEqual(r_local.object.unix_seconds, 42)
        self.assertEqual(r_local.pruned, 0)

        r_stranger = plan_backup_run("s3://b", "", SinkKind.S3, 9, 10_000, Retention(0), ("stranger.json", "backup-1.json"))
        self.assertEqual(r_stranger.pruned, 1)

        r_gcs = plan_backup_run("gs://b/backup", "backup", SinkKind.GCS, 5, 10_000, Retention(0), ("backup/backup-1.json",))
        self.assertEqual(r_gcs.object.key, "backup/backup-10000.json")
        self.assertEqual(r_gcs.pruned, 1)

    def test_run_result_to_json(self) -> None:
        res = BackupRunResult(
            object=BackupObject(sink="s3://b", key="backup-10000.json", bytes=128, unix_seconds=10_000),
            pruned=0,
        )
        expected = {
            "object": {
                "sink": "s3://b",
                "key": "backup-10000.json",
                "bytes": 128,
                "unixSeconds": 10_000,
            },
            "pruned": 0,
        }
        self.assertEqual(run_result_to_json(res), expected)


if __name__ == "__main__":
    unittest.main()
