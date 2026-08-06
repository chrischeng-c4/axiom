from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_backup.application.sink import (
    SinkKind,
    resolve_s3_region,
    select_sink,
    sink_identity,
    sink_prefix,
    unlinked_error,
)
from service_backup.domain.destination import Gcs, Local, S3, identity
from service_backup.domain.errors import UnlinkedAdapter, UnsupportedCredentialSecret
from service_backup.infrastructure.schemes import BuildFeatures


class TestApplicationSink(unittest.TestCase):
    def test_select_sink_local_and_gcs(self) -> None:
        f0 = BuildFeatures(s3=False)
        f1 = BuildFeatures(s3=True)
        self.assertEqual(select_sink(Local("/p"), f0), SinkKind.LOCAL)
        self.assertEqual(select_sink(Local("/p"), f1), SinkKind.LOCAL)
        self.assertEqual(select_sink(Gcs("b"), f0), SinkKind.GCS)

    def test_select_sink_gcs_credentials_unsupported(self) -> None:
        f0 = BuildFeatures(s3=False)
        self.assertEqual(
            select_sink(Gcs("b", credentials_secret="g"), f0),
            UnsupportedCredentialSecret("gs://b", "g"),
        )

    def test_select_sink_s3_adapter_check_before_credentials(self) -> None:
        f0 = BuildFeatures(s3=False)
        f1 = BuildFeatures(s3=True)

        self.assertEqual(select_sink(S3("b"), f0), SinkKind.UNSUPPORTED_CLOUD)
        self.assertEqual(select_sink(S3("b"), f1), SinkKind.S3)

        self.assertEqual(
            select_sink(S3("b", credentials_secret="x"), f0),
            SinkKind.UNSUPPORTED_CLOUD,
        )

        self.assertEqual(
            select_sink(S3("b", credentials_secret="x"), f1),
            UnsupportedCredentialSecret("s3://b", "x"),
        )
        self.assertEqual(
            select_sink(S3("b", prefix="p", credentials_secret="x"), f1),
            UnsupportedCredentialSecret("s3://b/p", "x"),
        )
        self.assertEqual(
            select_sink(S3("b", prefix="/nested/", credentials_secret="x"), f1),
            UnsupportedCredentialSecret("s3://b//nested/", "x"),
        )

    def test_unlinked_error(self) -> None:
        self.assertEqual(unlinked_error(S3("b")), UnlinkedAdapter("s3://b", "s3"))
        self.assertEqual(
            unlinked_error(S3("b", prefix="/nested/")),
            UnlinkedAdapter("s3://b//nested/", "s3"),
        )

    def test_sink_prefix(self) -> None:
        self.assertEqual(sink_prefix(Local("/p")), "backup")
        self.assertEqual(sink_prefix(Local("/p", prefix="")), "")
        self.assertEqual(sink_prefix(Local("/p", prefix="snap")), "snap")
        self.assertEqual(sink_prefix(S3("b")), "")
        self.assertEqual(sink_prefix(S3("b", prefix="p")), "p")
        self.assertEqual(sink_prefix(S3("b", prefix="/nested/prefix/")), "nested/prefix")
        self.assertEqual(sink_prefix(S3("b", prefix="/")), "")
        self.assertEqual(sink_prefix(Gcs("b")), "backup")
        self.assertEqual(sink_prefix(Gcs("b", prefix="p")), "p")
        self.assertEqual(sink_prefix(Gcs("b", prefix="/nested/")), "nested")
        self.assertEqual(sink_prefix(Gcs("b", prefix="/")), "")

    def test_sink_identity(self) -> None:
        self.assertEqual(sink_identity(Local("/p")), "local:/p")
        self.assertEqual(sink_identity(Local("/p", prefix="")), "local:/p")
        self.assertEqual(sink_identity(Local("/p", prefix="snap")), "local:/p")
        self.assertEqual(sink_identity(S3("b")), "s3://b")
        self.assertEqual(sink_identity(S3("b", prefix="p")), "s3://b/p")
        self.assertEqual(sink_identity(S3("b", prefix="/nested/prefix/")), "s3://b/nested/prefix")
        self.assertEqual(sink_identity(S3("b", prefix="/")), "s3://b")
        self.assertEqual(sink_identity(Gcs("b")), "gs://b/backup")
        self.assertEqual(sink_identity(Gcs("b", prefix="p")), "gs://b/p")
        self.assertEqual(sink_identity(Gcs("b", prefix="/nested/")), "gs://b/nested")
        self.assertEqual(sink_identity(Gcs("b", prefix="/")), "gs://b/")

    def test_sink_identity_vs_identity_disagreement(self) -> None:
        dest_gcs = Gcs("b")
        self.assertNotEqual(sink_identity(dest_gcs), identity(dest_gcs))
        self.assertEqual(sink_identity(dest_gcs), "gs://b/backup")
        self.assertEqual(identity(dest_gcs), "gs://b")

        dest_s3 = S3("b", prefix="/nested/prefix/")
        self.assertNotEqual(sink_identity(dest_s3), identity(dest_s3))
        self.assertEqual(sink_identity(dest_s3), "s3://b/nested/prefix")
        self.assertEqual(identity(dest_s3), "s3://b//nested/prefix/")

    def test_resolve_s3_region(self) -> None:
        self.assertIsNone(resolve_s3_region(S3("b")))
        self.assertEqual(resolve_s3_region(S3("b", region="eu-west-1")), "eu-west-1")
        self.assertEqual(resolve_s3_region(S3("b", endpoint="http://minio:9000")), "us-east-1")
        self.assertEqual(
            resolve_s3_region(S3("b", region="eu-west-1", endpoint="http://minio:9000")),
            "eu-west-1",
        )


if __name__ == "__main__":
    unittest.main()
