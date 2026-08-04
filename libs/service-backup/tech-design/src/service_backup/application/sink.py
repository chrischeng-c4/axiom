from __future__ import annotations

from enum import Enum

from service_backup.domain.destination import (
    DEFAULT_PREFIX,
    GCS_SCHEME,
    LOCAL_IDENTITY,
    S3_SCHEME,
    Destination,
    Gcs,
    Local,
    S3,
    default_prefix,
    identity,
)
from service_backup.domain.errors import (
    BackupError,
    UnlinkedAdapter,
    UnsupportedCredentialSecret,
)
from service_backup.infrastructure.keys import normalize_prefix
from service_backup.infrastructure.schemes import BuildFeatures

DEFAULT_S3_REGION = "us-east-1"


class SinkKind(str, Enum):
    LOCAL = "local"
    S3 = "s3"
    GCS = "gcs"
    UNSUPPORTED_CLOUD = "unsupported-cloud"


def select_sink(destination: Destination, features: BuildFeatures) -> SinkKind | BackupError:
    if isinstance(destination, Local):
        return SinkKind.LOCAL
    if isinstance(destination, Gcs):
        if destination.credentials_secret is not None:
            return UnsupportedCredentialSecret(identity(destination), destination.credentials_secret)
        return SinkKind.GCS
    if isinstance(destination, S3):
        if not features.s3:
            return SinkKind.UNSUPPORTED_CLOUD
        if destination.credentials_secret is not None:
            return UnsupportedCredentialSecret(identity(destination), destination.credentials_secret)
        return SinkKind.S3
    raise TypeError(f"Unknown destination type: {type(destination)}")


def unlinked_error(destination: Destination) -> UnlinkedAdapter:
    return UnlinkedAdapter(identity(destination), SinkKind.S3.value)


def sink_prefix(destination: Destination) -> str:
    if isinstance(destination, Local):
        return default_prefix(destination)
    if isinstance(destination, S3):
        return normalize_prefix(destination.prefix)
    if isinstance(destination, Gcs):
        if destination.prefix == "":
            return DEFAULT_PREFIX
        return normalize_prefix(destination.prefix)
    raise TypeError(f"Unknown destination type: {type(destination)}")


def sink_identity(destination: Destination) -> str:
    if isinstance(destination, Local):
        return LOCAL_IDENTITY + destination.path
    if isinstance(destination, S3):
        p = sink_prefix(destination)
        if p == "":
            return S3_SCHEME + destination.bucket
        return S3_SCHEME + destination.bucket + "/" + p
    if isinstance(destination, Gcs):
        return GCS_SCHEME + destination.bucket + "/" + sink_prefix(destination)
    raise TypeError(f"Unknown destination type: {type(destination)}")


def resolve_s3_region(destination: S3) -> str | None:
    if destination.region is not None:
        return destination.region
    if destination.endpoint is not None:
        return DEFAULT_S3_REGION
    return None
