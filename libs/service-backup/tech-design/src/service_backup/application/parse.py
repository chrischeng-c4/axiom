from __future__ import annotations

from service_backup.domain.destination import (
    FILE_SCHEME,
    GCS_SCHEME,
    S3_SCHEME,
    Destination,
    Gcs,
    Local,
    S3,
)
from service_backup.domain.errors import (
    BackupError,
    EmptyDestination,
    MissingBucket,
    MissingPath,
    UnsupportedScheme,
)
from service_backup.infrastructure.keys import normalize_prefix


def split_bucket_prefix(rest: str, scheme_label: str) -> tuple[str, str] | BackupError:
    rest = rest.rstrip("/")
    if "/" not in rest:
        if rest == "":
            return MissingBucket(scheme_label)
        return (rest, "")
    bucket, prefix = rest.split("/", 1)
    if bucket == "":
        return MissingBucket(scheme_label)
    return (bucket, normalize_prefix(prefix))


def parse_destination(raw_uri: str) -> Destination | BackupError:
    uri = raw_uri.strip()
    if uri == "":
        return EmptyDestination()

    if uri.startswith(FILE_SCHEME):
        path = uri[len(FILE_SCHEME) :]
        if path == "":
            return MissingPath("file")
        return Local(path=path)

    if uri.startswith(S3_SCHEME):
        r = split_bucket_prefix(uri[len(S3_SCHEME) :], "s3")
        if not isinstance(r, tuple):
            return r
        return S3(bucket=r[0], prefix=r[1])

    if uri.startswith(GCS_SCHEME):
        r = split_bucket_prefix(uri[len(GCS_SCHEME) :], "gs")
        if not isinstance(r, tuple):
            return r
        return Gcs(bucket=r[0], prefix=r[1])

    return UnsupportedScheme(uri, (FILE_SCHEME, S3_SCHEME, GCS_SCHEME))
