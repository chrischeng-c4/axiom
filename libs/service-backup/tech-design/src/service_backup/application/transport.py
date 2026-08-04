from __future__ import annotations

from service_backup.domain.errors import RemoteStatus

ADMIN_SNAPSHOT_PATH = "/admin/backup"
AUTHORIZATION_HEADER = "authorization"
BEARER_PREFIX = "Bearer "


def admin_snapshot_url(base_url: str) -> str:
    return base_url.rstrip("/") + ADMIN_SNAPSHOT_PATH


def admin_request_headers(token: str | None) -> dict[str, str]:
    if token is None:
        return {}
    return {AUTHORIZATION_HEADER: BEARER_PREFIX + token}


def classify_response(status: int, body: str) -> RemoteStatus | None:
    if 200 <= status < 300:
        return None
    return RemoteStatus(status, body)
