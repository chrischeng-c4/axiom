#!/usr/bin/env python3.12
"""Load and verify the exact typeshed corpus consumed by mamba generators."""

from __future__ import annotations

import hashlib
import json
import re
import tomllib
from dataclasses import dataclass
from pathlib import Path


MAMBA_DIR = Path(__file__).resolve().parents[4]
DEFAULT_LOCK_PATH = MAMBA_DIR / "vendor" / "typeshed.lock.toml"
_REVISION_RE = re.compile(r"[0-9a-f]{40}")
_SHA256_RE = re.compile(r"[0-9a-f]{64}")
_LOCK_FIELDS = {
    "schema",
    "repository",
    "revision",
    "stdlib_pyi_count",
    "stdlib_pyi_sha256",
}


class TypeshedLockError(ValueError):
    """The lock or the checked-out stdlib corpus is invalid."""


@dataclass(frozen=True)
class TypeshedLock:
    schema: int
    repository: str
    revision: str
    stdlib_pyi_count: int
    stdlib_pyi_sha256: str


def load_typeshed_lock(path: Path = DEFAULT_LOCK_PATH) -> TypeshedLock:
    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise TypeshedLockError(f"cannot read typeshed lock {path}: {error}") from error

    fields = set(data)
    if fields != _LOCK_FIELDS:
        missing = sorted(_LOCK_FIELDS - fields)
        extra = sorted(fields - _LOCK_FIELDS)
        raise TypeshedLockError(
            f"invalid typeshed lock fields: missing={missing} extra={extra}"
        )
    if type(data["schema"]) is not int or data["schema"] != 1:
        raise TypeshedLockError("typeshed lock schema must be integer 1")
    if not isinstance(data["repository"], str) or not data["repository"].startswith(
        ("https://", "ssh://", "git@")
    ):
        raise TypeshedLockError("typeshed lock repository must be a git URL")
    if not isinstance(data["revision"], str) or _REVISION_RE.fullmatch(
        data["revision"]
    ) is None:
        raise TypeshedLockError("typeshed lock revision must be 40 lowercase hex digits")
    if (
        type(data["stdlib_pyi_count"]) is not int
        or data["stdlib_pyi_count"] <= 0
    ):
        raise TypeshedLockError("typeshed lock stdlib_pyi_count must be positive")
    if not isinstance(data["stdlib_pyi_sha256"], str) or _SHA256_RE.fullmatch(
        data["stdlib_pyi_sha256"]
    ) is None:
        raise TypeshedLockError(
            "typeshed lock stdlib_pyi_sha256 must be 64 lowercase hex digits"
        )
    return TypeshedLock(**data)


def stdlib_pyi_fingerprint(stdlib: Path) -> tuple[int, str]:
    if not stdlib.is_dir():
        raise TypeshedLockError(f"missing typeshed stdlib directory: {stdlib}")
    paths = sorted(
        (path for path in stdlib.rglob("*.pyi") if path.is_file()),
        key=lambda path: path.relative_to(stdlib).as_posix(),
    )
    rows = [
        [
            path.relative_to(stdlib).as_posix(),
            hashlib.sha256(path.read_bytes()).hexdigest(),
        ]
        for path in paths
    ]
    payload = json.dumps(rows, ensure_ascii=True, separators=(",", ":")).encode()
    return len(rows), hashlib.sha256(payload).hexdigest()


def verify_typeshed_stdlib(
    stdlib: Path,
    *,
    lock: TypeshedLock | None = None,
    lock_path: Path = DEFAULT_LOCK_PATH,
) -> TypeshedLock:
    lock_source = "provided lock"
    if lock is None:
        lock = load_typeshed_lock(lock_path)
        lock_source = str(lock_path)
    count, digest = stdlib_pyi_fingerprint(stdlib)
    problems = []
    if count != lock.stdlib_pyi_count:
        problems.append(f"count expected={lock.stdlib_pyi_count} actual={count}")
    if digest != lock.stdlib_pyi_sha256:
        problems.append(
            f"sha256 expected={lock.stdlib_pyi_sha256} actual={digest}"
        )
    if problems:
        raise TypeshedLockError(
            f"typeshed stdlib does not match {lock_source}: " + "; ".join(problems)
        )
    return lock
