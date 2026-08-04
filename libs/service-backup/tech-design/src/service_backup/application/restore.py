from __future__ import annotations

from dataclasses import dataclass

from service_backup.domain.destination import FILE_SCHEME, GCS_SCHEME, S3_SCHEME
from service_backup.domain.errors import (
    BackupError,
    EmptyDestination,
    MissingBucket,
    MissingKey,
    MissingPath,
    UnlinkedAdapter,
    UnsupportedScheme,
)
from service_backup.infrastructure.schemes import BuildFeatures


@dataclass(frozen=True)
class LocalObject:
    path: str


@dataclass(frozen=True)
class RemoteObject:
    scheme: str
    bucket: str
    key: str


ObjectRef = LocalObject | RemoteObject


def split_bucket_key(rest: str, uri: str, scheme_label: str) -> tuple[str, str] | BackupError:
    if "/" not in rest:
        return MissingKey(uri)
    bucket, key = rest.split("/", 1)
    if bucket == "":
        return MissingBucket(scheme_label)
    key = key.lstrip("/")
    if key == "":
        return MissingKey(uri)
    return (bucket, key)


def parse_object_uri(raw_uri: str, features: BuildFeatures) -> ObjectRef | BackupError:
    uri = raw_uri.strip()
    if uri == "":
        return EmptyDestination()

    if uri.startswith(FILE_SCHEME):
        path = uri[len(FILE_SCHEME) :]
        if path == "":
            return MissingPath("file")
        return LocalObject(path=path)

    if uri.startswith(S3_SCHEME):
        r = split_bucket_key(uri[len(S3_SCHEME) :], uri, "s3")
        if not isinstance(r, tuple):
            return r
        if not features.s3:
            return UnlinkedAdapter(uri, "s3")
        return RemoteObject(scheme=S3_SCHEME, bucket=r[0], key=r[1])

    if uri.startswith(GCS_SCHEME):
        r = split_bucket_key(uri[len(GCS_SCHEME) :], uri, "gs")
        if not isinstance(r, tuple):
            return r
        return RemoteObject(scheme=GCS_SCHEME, bucket=r[0], key=r[1])

    return UnsupportedScheme(uri, (FILE_SCHEME, S3_SCHEME, GCS_SCHEME))
