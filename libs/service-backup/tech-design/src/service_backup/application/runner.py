from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass

from service_backup.application.sink import SinkKind
from service_backup.domain.policy import Retention, is_expired, prunes_by_age
from service_backup.infrastructure.keys import (
    build_key,
    local_object_name,
    normalize_prefix,
    parse_backup_key,
)


@dataclass(frozen=True)
class BackupObject:
    sink: str
    key: str
    bytes: int
    unix_seconds: int


@dataclass(frozen=True)
class BackupRunResult:
    object: BackupObject
    pruned: int


def object_key(kind: SinkKind, prefix: str, unix_seconds: int) -> str:
    if kind is SinkKind.LOCAL:
        return local_object_name(prefix, unix_seconds)
    if kind is SinkKind.S3 or kind is SinkKind.GCS:
        return build_key(normalize_prefix(prefix), unix_seconds)
    raise ValueError(f"Unsupported sink kind for key generation: {kind}")


def plan_prune(
    keys: Sequence[str],
    prefix: str,
    now_unix_seconds: int,
    retention: Retention,
) -> tuple[str, ...]:
    if not prunes_by_age(retention):
        return ()
    normalized = normalize_prefix(prefix)
    out: list[str] = []
    for key in keys:
        ts = parse_backup_key(normalized, key)
        if ts is None:
            continue
        if is_expired(ts, now_unix_seconds, retention):
            out.append(key)
    return tuple(out)


def plan_backup_run(
    sink_identity_value: str,
    prefix: str,
    kind: SinkKind,
    payload_size: int,
    unix_seconds: int,
    retention: Retention,
    existing_keys: Sequence[str] = (),
) -> BackupRunResult:
    key = object_key(kind, prefix, unix_seconds)
    candidates = tuple(existing_keys) + (key,)
    pruned = plan_prune(candidates, prefix, unix_seconds, retention)
    return BackupRunResult(
        object=BackupObject(
            sink=sink_identity_value,
            key=key,
            bytes=payload_size,
            unix_seconds=unix_seconds,
        ),
        pruned=len(pruned),
    )


def run_result_to_json(result: BackupRunResult) -> dict[str, object]:
    return {
        "object": {
            "sink": result.object.sink,
            "key": result.object.key,
            "bytes": result.object.bytes,
            "unixSeconds": result.object.unix_seconds,
        },
        "pruned": result.pruned,
    }
