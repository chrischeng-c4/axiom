from __future__ import annotations

from typing import Final, Union

SERVICE_LOG_SCHEMA_V1: Final[str] = "axiom.service.log.v1"
MAX_ATTRIBUTES: Final[int] = 64
MAX_ATTRIBUTE_KEY_BYTES: Final[int] = 128
MAX_ATTRIBUTE_VALUE_BYTES: Final[int] = 4096
MAX_EVENT_BYTES: Final[int] = 128
MAX_REQUEST_ID_BYTES: Final[int] = 128

JsonValue = Union[str, int, float, bool, None]
