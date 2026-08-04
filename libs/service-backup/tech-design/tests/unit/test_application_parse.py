from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_backup.application.parse import parse_destination, split_bucket_prefix
from service_backup.domain.destination import Gcs, Local, S3
from service_backup.domain.errors import (
    EmptyDestination,
    MissingBucket,
    MissingPath,
    UnsupportedScheme,
)


class TestApplicationParse(unittest.TestCase):
    def test_parse_destination_empty_and_whitespace(self) -> None:
        self.assertEqual(parse_destination(""), EmptyDestination())
        self.assertEqual(parse_destination("   "), EmptyDestination())

    def test_parse_destination_local_path(self) -> None:
        res = parse_destination("file:///tmp/backups")
        self.assertEqual(res, Local(path="/tmp/backups", prefix=None))

        res_space = parse_destination("  file:///tmp/b  ")
        self.assertEqual(res_space, Local(path="/tmp/b"))

        res_relative = parse_destination("file://relative")
        self.assertEqual(res_relative, Local(path="relative"))

        res_no_trim = parse_destination("file:///tmp/x/")
        self.assertEqual(res_no_trim, Local(path="/tmp/x/"))

    def test_parse_destination_local_missing_path(self) -> None:
        self.assertEqual(parse_destination("file://"), MissingPath("file"))

    def test_parse_destination_s3_bucket_and_prefix(self) -> None:
        self.assertEqual(parse_destination("s3://bucket"), S3(bucket="bucket", prefix=""))
        self.assertEqual(parse_destination("s3://bucket/"), S3(bucket="bucket", prefix=""))
        self.assertEqual(parse_destination("s3://bucket/a/b"), S3(bucket="bucket", prefix="a/b"))
        self.assertEqual(parse_destination("s3://bucket/a/b/"), S3(bucket="bucket", prefix="a/b"))
        self.assertEqual(parse_destination("s3://bucket//a//b//"), S3(bucket="bucket", prefix="a//b"))

    def test_parse_destination_s3_missing_bucket(self) -> None:
        self.assertEqual(parse_destination("s3://"), MissingBucket("s3"))
        self.assertEqual(parse_destination("s3:///"), MissingBucket("s3"))
        self.assertEqual(parse_destination("s3:///prefix"), MissingBucket("s3"))

    def test_parse_destination_gcs(self) -> None:
        self.assertEqual(parse_destination("gs://"), MissingBucket("gs"))
        self.assertEqual(parse_destination("gs://b/p"), Gcs(bucket="b", prefix="p"))

    def test_parse_destination_unsupported_scheme(self) -> None:
        schemes = ("file://", "s3://", "gs://")
        self.assertEqual(parse_destination("ftp://nope"), UnsupportedScheme("ftp://nope", schemes))
        self.assertEqual(parse_destination("FILE:///tmp"), UnsupportedScheme("FILE:///tmp", schemes))
        self.assertEqual(parse_destination("S3://b"), UnsupportedScheme("S3://b", schemes))
        self.assertEqual(parse_destination("/tmp/x"), UnsupportedScheme("/tmp/x", schemes))

    def test_split_bucket_prefix_direct(self) -> None:
        self.assertEqual(split_bucket_prefix("bucket", "s3"), ("bucket", ""))
        self.assertEqual(split_bucket_prefix("bucket/a/b", "s3"), ("bucket", "a/b"))
        self.assertEqual(split_bucket_prefix("", "s3"), MissingBucket("s3"))


if __name__ == "__main__":
    unittest.main()
