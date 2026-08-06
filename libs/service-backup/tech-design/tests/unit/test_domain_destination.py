from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_backup.domain.destination import Gcs, Local, S3, default_prefix, identity


class TestDomainDestination(unittest.TestCase):
    def test_local_identity(self) -> None:
        self.assertEqual(identity(Local(path="/tmp/backups")), "local:/tmp/backups")
        self.assertEqual(identity(Local(path="/tmp/backups", prefix="p")), "local:/tmp/backups")

    def test_s3_identity(self) -> None:
        self.assertEqual(identity(S3(bucket="b")), "s3://b")
        self.assertEqual(identity(S3(bucket="b", prefix="a/b")), "s3://b/a/b")
        self.assertEqual(
            identity(S3(bucket="b", prefix="p", region="r", endpoint="e", credentials_secret="s")),
            "s3://b/p",
        )

    def test_gcs_identity(self) -> None:
        self.assertEqual(identity(Gcs(bucket="b")), "gs://b")
        self.assertEqual(identity(Gcs(bucket="b", prefix="prefix")), "gs://b/prefix")

    def test_local_default_prefix(self) -> None:
        self.assertEqual(default_prefix(Local(path="/p")), "backup")
        self.assertEqual(default_prefix(Local(path="/p", prefix="")), "")
        self.assertEqual(default_prefix(Local(path="/p", prefix="x")), "x")

    def test_s3_default_prefix(self) -> None:
        self.assertEqual(default_prefix(S3(bucket="b")), "backup")
        self.assertEqual(default_prefix(S3(bucket="b", prefix="")), "backup")
        self.assertEqual(default_prefix(S3(bucket="b", prefix="x")), "x")

    def test_gcs_default_prefix(self) -> None:
        self.assertEqual(default_prefix(Gcs(bucket="b")), "backup")
        self.assertEqual(default_prefix(Gcs(bucket="b", prefix="")), "backup")
        self.assertEqual(default_prefix(Gcs(bucket="b", prefix="x")), "x")

    def test_prefix_fallback_asymmetry(self) -> None:
        self.assertEqual(default_prefix(Local(path="/p", prefix="")), "")
        self.assertEqual(default_prefix(S3(bucket="b", prefix="")), "backup")


if __name__ == "__main__":
    unittest.main()
