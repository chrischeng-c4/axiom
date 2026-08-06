from __future__ import annotations

from service_backup.domain.policy import Retention, ScheduledBackupPolicy


def retention_to_json(retention: Retention) -> dict[str, object]:
    if retention.max_age_seconds is None:
        return {}
    return {"maxAgeSeconds": retention.max_age_seconds}


def scheduled_policy_to_json(
    policy: ScheduledBackupPolicy,
) -> dict[str, object]:
    body: dict[str, object] = {
        "schedule": policy.schedule,
        "destination": policy.destination,
    }
    if policy.retention_secs is not None:
        body["retentionSecs"] = policy.retention_secs
    return body


def scheduled_policy_schema() -> dict[str, object]:
    return {
        "type": "object",
        "required": ["schedule", "destination"],
        "properties": {
            "schedule": {"type": "string"},
            "destination": {"type": "string"},
            "retentionSecs": {"type": "integer", "minimum": 0},
        },
    }


def is_structural(schema: object) -> bool:
    if isinstance(schema, dict):
        for key in ("oneOf", "anyOf", "allOf"):
            if key in schema:
                return False
        for value in schema.values():
            if not is_structural(value):
                return False
        return True
    if isinstance(schema, list):
        for item in schema:
            if not is_structural(item):
                return False
        return True
    return True
