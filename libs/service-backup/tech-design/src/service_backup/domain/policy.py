from __future__ import annotations

from dataclasses import dataclass, field

from service_backup.domain.destination import Destination


@dataclass(frozen=True)
class Retention:
    max_age_seconds: int | None = None


@dataclass(frozen=True)
class BackupPolicy:
    schedule: str
    destination: Destination
    retention: Retention = field(default_factory=Retention)


@dataclass(frozen=True)
class ScheduledBackupPolicy:
    schedule: str
    destination: str
    retention_secs: int | None = None


def is_blank_schedule(schedule: str) -> bool:
    return schedule.strip() == ""


def prunes_by_age(retention: Retention) -> bool:
    return retention.max_age_seconds is not None


def is_expired(
    object_unix_seconds: int,
    now_unix_seconds: int,
    retention: Retention,
) -> bool:
    if retention.max_age_seconds is None:
        return False
    cutoff = now_unix_seconds - retention.max_age_seconds
    return object_unix_seconds < cutoff
