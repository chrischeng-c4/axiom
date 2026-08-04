from __future__ import annotations

from dataclasses import dataclass

from service_backup.domain.destination import FILE_SCHEME, GCS_SCHEME, S3_SCHEME


@dataclass(frozen=True)
class SchemeInfo:
    scheme: str
    description: str
    sink_available: bool


@dataclass(frozen=True)
class BuildFeatures:
    s3: bool = False


def supported_schemes(features: BuildFeatures) -> tuple[SchemeInfo, ...]:
    return (
        SchemeInfo(
            FILE_SCHEME,
            "local filesystem path - dev/tests and PVC-backed local runs",
            True,
        ),
        SchemeInfo(
            S3_SCHEME,
            "Amazon S3-compatible object store",
            features.s3,
        ),
        SchemeInfo(
            GCS_SCHEME,
            "Google Cloud Storage - workload identity in production",
            True,
        ),
    )


def scheme_names(features: BuildFeatures) -> tuple[str, ...]:
    return tuple(s.scheme for s in supported_schemes(features))


def find_scheme(scheme: str, features: BuildFeatures) -> SchemeInfo | None:
    for entry in supported_schemes(features):
        if entry.scheme == scheme:
            return entry
    return None


def unavailable_schemes(features: BuildFeatures) -> tuple[str, ...]:
    return tuple(s.scheme for s in supported_schemes(features) if not s.sink_available)


def topic_destination_section(features: BuildFeatures) -> str:
    lines: list[str] = []
    for entry in supported_schemes(features):
        if entry.sink_available:
            lines.append(f"{entry.scheme}  {entry.description}")
        else:
            lines.append(f"{entry.scheme}  {entry.description} (not linked in this build)")
    return "\n".join(lines)
