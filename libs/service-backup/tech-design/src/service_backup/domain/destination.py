from __future__ import annotations

from dataclasses import dataclass

FILE_SCHEME = "file://"
S3_SCHEME = "s3://"
GCS_SCHEME = "gs://"
LOCAL_IDENTITY = "local:"
DEFAULT_PREFIX = "backup"


@dataclass(frozen=True)
class Local:
    path: str
    prefix: str | None = None


@dataclass(frozen=True)
class S3:
    bucket: str
    prefix: str = ""
    region: str | None = None
    endpoint: str | None = None
    credentials_secret: str | None = None


@dataclass(frozen=True)
class Gcs:
    bucket: str
    prefix: str = ""
    credentials_secret: str | None = None


Destination = Local | S3 | Gcs


def identity(destination: Destination) -> str:
    if isinstance(destination, Local):
        return LOCAL_IDENTITY + destination.path
    if isinstance(destination, S3):
        if destination.prefix == "":
            return S3_SCHEME + destination.bucket
        return S3_SCHEME + destination.bucket + "/" + destination.prefix
    if isinstance(destination, Gcs):
        if destination.prefix == "":
            return GCS_SCHEME + destination.bucket
        return GCS_SCHEME + destination.bucket + "/" + destination.prefix
    raise TypeError(f"Unknown destination type: {type(destination)}")


def default_prefix(destination: Destination) -> str:
    if isinstance(destination, Local):
        return destination.prefix if destination.prefix is not None else DEFAULT_PREFIX
    if isinstance(destination, S3):
        return DEFAULT_PREFIX if destination.prefix == "" else destination.prefix
    if isinstance(destination, Gcs):
        return DEFAULT_PREFIX if destination.prefix == "" else destination.prefix
    raise TypeError(f"Unknown destination type: {type(destination)}")
