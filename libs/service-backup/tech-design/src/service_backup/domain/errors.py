from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class EmptyDestination:
    pass


@dataclass(frozen=True)
class MissingPath:
    scheme: str


@dataclass(frozen=True)
class MissingBucket:
    scheme: str


@dataclass(frozen=True)
class MissingKey:
    uri: str


@dataclass(frozen=True)
class UnsupportedScheme:
    uri: str
    supported: tuple[str, ...]


@dataclass(frozen=True)
class EmptySchedule:
    pass


@dataclass(frozen=True)
class UnlinkedAdapter:
    destination: str
    feature: str


@dataclass(frozen=True)
class UnsupportedCredentialSecret:
    destination: str
    secret: str


@dataclass(frozen=True)
class RemoteStatus:
    status: int
    body: str


BackupError = (
    EmptyDestination
    | MissingPath
    | MissingBucket
    | MissingKey
    | UnsupportedScheme
    | EmptySchedule
    | UnlinkedAdapter
    | UnsupportedCredentialSecret
    | RemoteStatus
)


def describe(error: BackupError) -> str:
    if isinstance(error, EmptyDestination):
        return "backup destination URI is empty"
    if isinstance(error, MissingPath):
        return f"{error.scheme} backup URI has no path"
    if isinstance(error, MissingBucket):
        return f"{error.scheme} backup URI has no bucket"
    if isinstance(error, MissingKey):
        return f"backup object URI `{error.uri}` has no object key"
    if isinstance(error, UnsupportedScheme):
        supported_str = ", ".join(error.supported)
        return f"unsupported backup destination URI `{error.uri}`; use {supported_str}"
    if isinstance(error, EmptySchedule):
        return "backup schedule must not be empty"
    if isinstance(error, UnlinkedAdapter):
        return (
            f"backup destination {error.destination} needs the `{error.feature}` feature; "
            f"rebuild with --features {error.feature} or use a local destination"
        )
    if isinstance(error, UnsupportedCredentialSecret):
        return (
            f"backup destination {error.destination} sets credentials_secret `{error.secret}`, "
            "but secret-mounted credentials are not implemented; "
            "use ambient credentials or omit credentials_secret"
        )
    if isinstance(error, RemoteStatus):
        return f"admin snapshot request failed with status {error.status}: {error.body}"
    raise TypeError(f"Unknown BackupError variant: {type(error)}")
