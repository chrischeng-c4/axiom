from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_backup.application.restore import LocalObject, RemoteObject, parse_object_uri
from service_backup.domain.errors import (
    EmptyDestination,
    MissingBucket,
    MissingKey,
    MissingPath,
    UnlinkedAdapter,
    UnsupportedScheme,
)
from service_backup.infrastructure.schemes import BuildFeatures


class TestApplicationRestore(unittest.TestCase):
    def test_parse_object_uri_empty(self) -> None:
        f1 = BuildFeatures(s3=True)
        self.assertEqual(parse_object_uri("", f1), EmptyDestination())

    def test_parse_object_uri_local(self) -> None:
        f0 = BuildFeatures(s3=False)
        self.assertEqual(parse_object_uri("file:///tmp/s.json", f0), LocalObject(path="/tmp/s.json"))
        self.assertEqual(parse_object_uri("file://", f0), MissingPath("file"))

    def test_parse_object_uri_s3_success(self) -> None:
        f1 = BuildFeatures(s3=True)
        expected = RemoteObject(scheme="s3://", bucket="b", key="a/s.json")
        self.assertEqual(parse_object_uri("s3://b/a/s.json", f1), expected)

    def test_parse_object_uri_s3_unlinked(self) -> None:
        f0 = BuildFeatures(s3=False)
        self.assertEqual(
            parse_object_uri("s3://b/a/s.json", f0),
            UnlinkedAdapter("s3://b/a/s.json", "s3"),
        )

    def test_parse_object_uri_s3_grammar_before_adapter(self) -> None:
        f0 = BuildFeatures(s3=False)
        self.assertEqual(parse_object_uri("s3://b", f0), MissingKey("s3://b"))
        self.assertEqual(parse_object_uri("s3:///k", f0), MissingBucket("s3"))

    def test_parse_object_uri_s3_keys_and_slashes(self) -> None:
        f1 = BuildFeatures(s3=True)
        self.assertEqual(parse_object_uri("s3://b", f1), MissingKey("s3://b"))
        self.assertEqual(parse_object_uri("s3://b/", f1), MissingKey("s3://b/"))
        self.assertEqual(parse_object_uri("s3://b//", f1), MissingKey("s3://b//"))
        self.assertEqual(
            parse_object_uri("s3://b//a.json", f1),
            RemoteObject(scheme="s3://", bucket="b", key="a.json"),
        )
        self.assertEqual(
            parse_object_uri("s3://b/a//", f1),
            RemoteObject(scheme="s3://", bucket="b", key="a//"),
        )
        self.assertEqual(
            parse_object_uri("  s3://b/k  ", f1),
            RemoteObject(scheme="s3://", bucket="b", key="k"),
        )
        self.assertEqual(parse_object_uri("  s3://b  ", f1), MissingKey("s3://b"))

    def test_parse_object_uri_gcs_always_linked(self) -> None:
        f0 = BuildFeatures(s3=False)
        self.assertEqual(
            parse_object_uri("gs://b/k", f0),
            RemoteObject(scheme="gs://", bucket="b", key="k"),
        )
        self.assertEqual(parse_object_uri("gs://b", f0), MissingKey("gs://b"))
        self.assertEqual(parse_object_uri("gs:///k", f0), MissingBucket("gs"))

    def test_parse_object_uri_unsupported(self) -> None:
        f1 = BuildFeatures(s3=True)
        schemes = ("file://", "s3://", "gs://")
        self.assertEqual(parse_object_uri("ftp://x", f1), UnsupportedScheme("ftp://x", schemes))


if __name__ == "__main__":
    unittest.main()
