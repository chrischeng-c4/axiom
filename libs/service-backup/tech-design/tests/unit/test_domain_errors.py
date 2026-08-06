from __future__ import annotations

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "src"))

from service_backup.domain.errors import (
    EmptyDestination,
    EmptySchedule,
    MissingBucket,
    MissingKey,
    MissingPath,
    RemoteStatus,
    UnlinkedAdapter,
    UnsupportedCredentialSecret,
    UnsupportedScheme,
    describe,
)


class TestDomainErrors(unittest.TestCase):
    def test_describe_empty_destination(self) -> None:
        self.assertEqual(describe(EmptyDestination()), "backup destination URI is empty")

    def test_describe_missing_path(self) -> None:
        self.assertEqual(describe(MissingPath("file")), "file backup URI has no path")

    def test_describe_missing_bucket(self) -> None:
        self.assertEqual(describe(MissingBucket("s3")), "s3 backup URI has no bucket")

    def test_describe_missing_key(self) -> None:
        self.assertEqual(
            describe(MissingKey("s3://b")),
            "backup object URI `s3://b` has no object key",
        )

    def test_describe_unsupported_scheme_multiple(self) -> None:
        err = UnsupportedScheme("ftp://x", ("file://", "s3://", "gs://"))
        self.assertEqual(
            describe(err),
            "unsupported backup destination URI `ftp://x`; use file://, s3://, gs://",
        )

    def test_describe_unsupported_scheme_single(self) -> None:
        err = UnsupportedScheme("ftp://x", ("gs://",))
        self.assertEqual(
            describe(err),
            "unsupported backup destination URI `ftp://x`; use gs://",
        )

    def test_describe_unsupported_scheme_order_preserved(self) -> None:
        err = UnsupportedScheme("ftp://x", ("b://", "a://"))
        self.assertEqual(
            describe(err),
            "unsupported backup destination URI `ftp://x`; use b://, a://",
        )

    def test_describe_empty_schedule(self) -> None:
        self.assertEqual(describe(EmptySchedule()), "backup schedule must not be empty")

    def test_describe_unlinked_adapter(self) -> None:
        err = UnlinkedAdapter("s3://b/p", "s3")
        desc = describe(err)
        self.assertIn("s3://b/p", desc)
        self.assertIn("`s3`", desc)
        self.assertIn("--features s3", desc)
        self.assertEqual(
            desc,
            "backup destination s3://b/p needs the `s3` feature; rebuild with --features s3 or use a local destination",
        )

    def test_describe_unsupported_credential_secret(self) -> None:
        err = UnsupportedCredentialSecret("s3://b/p", "aws-creds")
        desc = describe(err)
        self.assertIn("s3://b/p", desc)
        self.assertIn("aws-creds", desc)
        self.assertIn("credentials_secret", desc)
        self.assertEqual(
            desc,
            "backup destination s3://b/p sets credentials_secret `aws-creds`, but secret-mounted credentials are not implemented; use ambient credentials or omit credentials_secret",
        )

    def test_describe_remote_status(self) -> None:
        err = RemoteStatus(403, "denied")
        self.assertEqual(
            describe(err),
            "admin snapshot request failed with status 403: denied",
        )


if __name__ == "__main__":
    unittest.main()
