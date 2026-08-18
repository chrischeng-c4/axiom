from __future__ import annotations

OBJECT_NAME_PREFIX = "backup-"
OBJECT_NAME_SUFFIX = ".json"


def normalize_prefix(prefix: str) -> str:
    return prefix.strip("/")


def build_key(prefix: str, unix_seconds: int) -> str:
    name = OBJECT_NAME_PREFIX + str(unix_seconds) + OBJECT_NAME_SUFFIX
    if prefix == "":
        return name
    return prefix + "/" + name


def parse_backup_key(prefix: str, key: str) -> int | None:
    if prefix == "":
        name = key
    else:
        if not key.startswith(prefix):
            return None
        rest = key[len(prefix) :]
        if not rest.startswith("/"):
            return None
        name = rest[1:]
    if not name.startswith(OBJECT_NAME_PREFIX):
        return None
    if not name.endswith(OBJECT_NAME_SUFFIX):
        return None
    digits = name[len(OBJECT_NAME_PREFIX) : -len(OBJECT_NAME_SUFFIX)]
    if digits == "":
        return None
    if not all(c in "0123456789" for c in digits):
        return None
    return int(digits)


def local_object_name(prefix: str, unix_seconds: int) -> str:
    return prefix + "-" + str(unix_seconds) + OBJECT_NAME_SUFFIX


def list_prefix(prefix: str) -> str | None:
    if prefix == "":
        return None
    return prefix + "/"
