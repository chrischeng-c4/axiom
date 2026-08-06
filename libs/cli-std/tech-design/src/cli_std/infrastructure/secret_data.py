from __future__ import annotations

import base64
import binascii
from collections.abc import Mapping
from dataclasses import dataclass

TOKEN_REGISTRY_SECRET_KEY: str = "token-registry.json"


@dataclass(frozen=True)
class MissingDataKey:
    key: str


@dataclass(frozen=True)
class NotBase64:
    key: str


SecretError = MissingDataKey | NotBase64


def secret_data_bytes(secret_json: object, key: str) -> bytes | SecretError:
    if not isinstance(secret_json, Mapping):
        return MissingDataKey(key)

    data = secret_json.get("data")
    if not isinstance(data, Mapping):
        return MissingDataKey(key)

    encoded = data.get(key)
    if not isinstance(encoded, str):
        return MissingDataKey(key)

    try:
        return base64.b64decode(encoded, validate=True)
    except (binascii.Error, ValueError):
        return NotBase64(key)


def cr_tokens_secret(cr_json: object) -> str | None:
    if not isinstance(cr_json, Mapping):
        return None
    spec = cr_json.get("spec")
    if not isinstance(spec, Mapping):
        return None
    tok_sec = spec.get("tokensSecret")
    if isinstance(tok_sec, str):
        return tok_sec
    return None
