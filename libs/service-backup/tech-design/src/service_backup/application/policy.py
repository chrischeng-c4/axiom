from __future__ import annotations

from service_backup.application.parse import parse_destination
from service_backup.domain.destination import Gcs, Local, S3
from service_backup.domain.errors import BackupError, EmptySchedule
from service_backup.domain.policy import (
    BackupPolicy,
    Retention,
    ScheduledBackupPolicy,
    is_blank_schedule,
)


def to_runtime_policy(scheduled: ScheduledBackupPolicy) -> BackupPolicy | BackupError:
    if is_blank_schedule(scheduled.schedule):
        return EmptySchedule()
    destination = parse_destination(scheduled.destination)
    if not isinstance(destination, (Local, S3, Gcs)):
        return destination
    return BackupPolicy(
        schedule=scheduled.schedule,
        destination=destination,
        retention=Retention(scheduled.retention_secs),
    )
